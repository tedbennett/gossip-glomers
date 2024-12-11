use gossip_glomers::*;

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
    ) -> anyhow::Result<Option<Message<EchoPayload>>> {
        let mut reply = input.into_reply(Some(self.id));
        let message = match reply.body.payload {
            EchoPayload::Echo { echo } => {
                reply.body.payload = EchoPayload::EchoOk { echo };
                self.id += 1;
                Some(reply)
            }
            EchoPayload::EchoOk { .. } => None,
        };
        Ok(message)
    }

    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<EchoPayload>>> {
        unreachable!()
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, EchoNode>()
}
