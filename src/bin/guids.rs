use gossip_glomers::*;

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
    ) -> anyhow::Result<Option<Message<GeneratePayload>>> {
        let mut reply = input.into_reply(Some(self.id));
        let message = match reply.body.payload {
            GeneratePayload::Generate => {
                let guid = format!("{}-{}", self.node_id, self.id);
                reply.body.payload = GeneratePayload::GenerateOk { id: guid };
                self.id += 1;
                Some(reply)
            }
            GeneratePayload::GenerateOk { .. } => None,
        };
        Ok(message)
    }

    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<GeneratePayload>>> {
        unreachable!()
    }
}

fn main() -> anyhow::Result<()> {
    run_loop::<_, GenerateNode>()
}
