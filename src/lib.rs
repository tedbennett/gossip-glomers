use std::{
    io::{BufRead, StdoutLock, Write},
    time::Duration,
};

use anyhow::{bail, Context};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::Instant,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

impl<Payload> Message<Payload> {
    pub fn into_reply(self, id: Option<usize>) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                id,
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Event<Payload> {
    Message(Message<Payload>),
    Gossip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<Payload> {
    fn new(id: usize, init: Init) -> Self;
    fn handle(&mut self, input: Message<Payload>) -> anyhow::Result<Option<Message<Payload>>>;
    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<Payload>>>;
}

fn handle_init(
    message: Message<InitPayload>,
    output: &mut StdoutLock,
) -> anyhow::Result<(usize, Init)> {
    let InitPayload::Init(init) = message.body.payload else {
        bail!("first message was invalid init payload")
    };
    let reply = Message {
        src: message.dest,
        dest: message.src,
        body: Body {
            id: Some(0),
            in_reply_to: message.body.id,
            payload: InitPayload::InitOk,
        },
    };
    serde_json::to_writer(&mut *output, &reply).context("serializing init reply")?;
    output.write_all(b"\n").context("flushing init reply")?;
    Ok((1, init))
}

pub fn send<P: Serialize>(message: &Message<P>, output: &mut StdoutLock) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *output, message).context("serializing broadcast reply")?;
    output
        .write_all(b"\n")
        .context("flushing broadcast reply")?;
    Ok(())
}

pub fn run_loop<P: DeserializeOwned + Serialize, N: Node<P>>() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    // Parse first message from stdin
    let first_msg = stdin.next().expect("did not receive init message")?;
    let init_msg: Message<InitPayload> =
        serde_json::from_str(&first_msg).context("failed to deserialize init message")?;

    // Retrieve init payload, send response, and return payload
    let (msg_id, init) = handle_init(init_msg, &mut stdout)?;

    let mut state = N::new(msg_id, init);

    for line in stdin {
        let line = line.context("failed to read next line")?;
        let input: Message<P> =
            serde_json::from_str(&line).context("failed to deserialize message")?;
        let message = state.handle(input).context("handling message failed")?;
        if let Some(message) = message {
            send(&message, &mut stdout)?;
        }
    }
    Ok(())
}

pub async fn async_run_loop<P: DeserializeOwned + Serialize + Send + 'static, N: Node<P>>(
) -> anyhow::Result<()> {
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    let mut stdout = std::io::stdout().lock();

    // Parse first message from stdin
    let first_msg = stdin
        .next_line()
        .await
        .expect("did not receive init message")
        .context("failed to retrieve first message")?;
    let init_msg: Message<InitPayload> =
        serde_json::from_str(&first_msg).context("failed to deserialize init message")?;

    // Retrieve init payload, send response, and return payload
    let (msg_id, init) = handle_init(init_msg, &mut stdout)?;

    let mut state = N::new(msg_id, init);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event<P>>(100);
    let stdin_tx = tx.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdin.next_line().await {
            let Ok(input) = serde_json::from_str(&line) else {
                return;
            };
            let _ = stdin_tx.send(Event::Message(input)).await;
        }
    });

    tokio::spawn(async move {
        // Needs to start after an interval so nodes just starting up don't receive a gossip first
        let mut timer = tokio::time::interval_at(
            Instant::now() + Duration::from_millis(300),
            Duration::from_millis(300),
        );
        loop {
            _ = timer.tick().await;
            let _ = tx.send(Event::Gossip).await;
        }
    });

    while let Some(event) = rx.recv().await {
        match event {
            Event::Message(message) => {
                let message = state.handle(message).context("handling message failed")?;
                if let Some(message) = message {
                    send(&message, &mut stdout)?;
                }
            }
            Event::Gossip => {
                let messages = state.handle_gossip().context("handling gossip failed")?;
                for message in messages {
                    send(&message, &mut stdout)?;
                }
            }
        };
    }
    Ok(())
}
