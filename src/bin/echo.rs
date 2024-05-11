use gossip_glomers::*;
use std::io::{StdoutLock, Write};

use anyhow::Context;
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
        match input.body.payload {
            EchoPayload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: EchoPayload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply).context("serializing echo reply")?;
                output.write_all(b"\n").context("flushing echo reply")?;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, EchoNode>()
}
