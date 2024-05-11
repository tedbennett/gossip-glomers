use gossip_glomers::*;
use std::{collections::HashMap, io::StdoutLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast { message: usize },
    BroadcastOk,
    Read,
    ReadOk { messages: Vec<usize> },
    Topology { topology: Topology },
    TopologyOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Topology {
    #[serde(flatten)]
    neighbours: HashMap<String, Vec<String>>,
}

struct BroadcastNode {
    id: usize,
    node_id: String,
    messages: Vec<usize>,
    neighbours: Vec<String>,
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            node_id: init.node_id,
            messages: Vec::new(),
            neighbours: Vec::new(),
        }
    }

    fn handle(
        &mut self,
        input: Message<BroadcastPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match &input.body.payload {
            BroadcastPayload::Broadcast { message } => {
                // Store in list of messages
                self.messages.push(*message);
                let reply = self.get_reply(BroadcastPayload::BroadcastOk, input)?;
                send(reply, output)?;
            }
            BroadcastPayload::BroadcastOk { .. } => {}
            BroadcastPayload::Read => {
                let reply = self.get_reply(
                    BroadcastPayload::ReadOk {
                        messages: self.messages.clone(),
                    },
                    input,
                )?;
                send(reply, output)?;
            }
            BroadcastPayload::ReadOk { .. } => {}
            BroadcastPayload::Topology { topology } => {
                if let Some(neighbours) = topology.neighbours.get(&self.node_id) {
                    self.neighbours = neighbours.to_vec();
                }
                let reply = self.get_reply(BroadcastPayload::TopologyOk, input)?;
                send(reply, output)?;
            }
            BroadcastPayload::TopologyOk => {}
        }
        Ok(())
    }

    fn get_reply(
        &mut self,
        response: BroadcastPayload,
        input: Message<BroadcastPayload>,
    ) -> anyhow::Result<Message<BroadcastPayload>> {
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
    run_loop::<_, BroadcastNode>()
}
