use gossip_glomers::*;
use std::io::{StdoutLock, Write};

use anyhow::Context;
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
        match &input.body.payload {
            GeneratePayload::Generate => {
                let reply = self.get_reply(
                    GeneratePayload::GenerateOk {
                        id: format!("{}-{}", self.node_id, self.id),
                    },
                    input,
                )?;
                send(reply, output)?;
            }
            GeneratePayload::GenerateOk { .. } => {}
        }
        Ok(())
    }

    fn get_reply(
        &mut self,
        response: GeneratePayload,
        input: Message<GeneratePayload>,
    ) -> anyhow::Result<Message<GeneratePayload>> {
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
    run_loop::<_, GenerateNode>()
}
