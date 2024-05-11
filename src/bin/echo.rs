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
        let mut reply = input.into_reply(Some(self.id));
        match reply.body.payload {
            EchoPayload::Echo { echo } => {
                reply.body.payload = EchoPayload::EchoOk { echo };
                send(&reply, output)?;
                self.id += 1;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }

    fn handle_gossip(&self, _output: &mut StdoutLock) -> anyhow::Result<()> {
        unreachable!()
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, EchoNode>()
}
