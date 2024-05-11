use std::io::{BufRead, StdoutLock, Write};

use anyhow::{bail, Context};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<Payload> {
    fn new(id: usize, init: Init) -> Self;
    fn handle(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
    fn get_reply(
        &mut self,
        response: Payload,
        input: Message<Payload>,
    ) -> anyhow::Result<Message<Payload>>;
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

pub fn send<P: Serialize>(message: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *output, &message).context("serializing broadcast reply")?;
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
        state
            .handle(input, &mut stdout)
            .context("handling message failed")?;
    }
    Ok(())
}
