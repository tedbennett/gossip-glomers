use gossip_glomers::*;
use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: usize,
    _node_id: String,
}

impl Node<EchoPayload> for EchoNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            _node_id: init.node_id,
        }
    }
    fn handle(
        &mut self,
        input: Message<EchoPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match &input.body.payload {
            EchoPayload::Echo { echo } => {
                let reply = self.get_reply(
                    EchoPayload::EchoOk {
                        echo: echo.to_string(),
                    },
                    input,
                )?;
                send(reply, output)?;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }

    fn get_reply(
        &mut self,
        response: EchoPayload,
        input: Message<EchoPayload>,
    ) -> anyhow::Result<Message<EchoPayload>> {
        let reply = Message {
            src: input.dest,
            dest: input.src,
            body: Body {
                id: Some(self.id),
                in_reply_to: input.body.id,
                payload: response,
            },
        };
        self.id += 1;
        Ok(reply)
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, EchoNode>()
}
