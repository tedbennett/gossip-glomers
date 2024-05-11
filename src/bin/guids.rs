use gossip_glomers::*;
use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum GeneratePayload {
    Generate,
    GenerateOk { id: String },
}

struct GenerateNode {
    id: usize,
    node_id: String,
}

impl Node<GeneratePayload> for GenerateNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            node_id: init.node_id,
        }
    }
    fn handle(
        &mut self,
        input: Message<GeneratePayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(self.id));
        match reply.body.payload {
            GeneratePayload::Generate => {
                let guid = format!("{}-{}", self.node_id, self.id);
                reply.body.payload = GeneratePayload::GenerateOk { id: guid };
                send(&reply, output)?;
                self.id += 1;
            }
            GeneratePayload::GenerateOk { .. } => {}
        }
        Ok(())
    }

    fn handle_gossip(&self, _output: &mut StdoutLock) -> anyhow::Result<()> {
        unreachable!()
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, GenerateNode>()
}
