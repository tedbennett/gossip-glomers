use gossip_glomers::*;
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum GCounterPayload {
    Add { delta: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
    Topology { topology: Topology },
    TopologyOk,
    Gossip { messages: Vec<Increment> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Increment {
    msg_id: String,
    delta: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Topology {
    #[serde(flatten)]
    neighbours: HashMap<String, Vec<String>>,
}

struct GCounterNode {
    id: usize,
    node_id: String,
    // Map of message id to increment value
    messages: HashMap<String, usize>,
    known_messages: HashMap<String, HashSet<String>>,
    neighbours: Vec<String>,
}

impl Node<GCounterPayload> for GCounterNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            neighbours: init
                .node_ids
                .clone()
                .into_iter()
                .filter(|n| n != &init.node_id)
                .collect(),
            node_id: init.node_id,
            messages: HashMap::new(),
            known_messages: HashMap::from_iter(
                init.node_ids.into_iter().map(|n| (n, HashSet::new())),
            ),
        }
    }

    fn handle(
        &mut self,
        input: Message<GCounterPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(self.id));
        match reply.body.payload {
            GCounterPayload::Add { delta } => {
                // Store in list of messages
                if let Some(id) = reply.body.id {
                    self.messages
                        .insert(format!("{}-{}", self.node_id, id), delta);
                }
                reply.body.payload = GCounterPayload::AddOk;
                send(&reply, output)?;
                self.id += 1;
            }
            GCounterPayload::Read => {
                reply.body.payload = GCounterPayload::ReadOk {
                    value: self.messages.values().into_iter().sum(),
                };

                send(&reply, output)?;
                self.id += 1;
            }
            GCounterPayload::Topology { topology } => {
                if let Some(neighbours) = topology.neighbours.get(&self.node_id) {
                    self.neighbours = neighbours
                        .to_vec()
                        .into_iter()
                        .filter(|n| n != &self.node_id)
                        .collect();
                }
                reply.body.payload = GCounterPayload::TopologyOk;
                send(&reply, output)?;
                self.id += 1;
            }
            GCounterPayload::Gossip { messages } => {
                for message in messages {
                    self.messages.insert(message.msg_id, message.delta);
                }
            }
            GCounterPayload::TopologyOk
            | GCounterPayload::ReadOk { .. }
            | GCounterPayload::AddOk { .. } => {}
        }
        Ok(())
    }

    fn handle_gossip(&self, output: &mut StdoutLock) -> anyhow::Result<()> {
        for neighbour in self.neighbours.as_slice() {
            let known = &self.known_messages[neighbour];
            let messages = self
                .messages
                .clone()
                .into_iter()
                .filter(|m| !known.contains(&m.0))
                .map(|m| Increment {
                    msg_id: m.0,
                    delta: m.1,
                })
                .collect();
            let message = Message {
                src: self.node_id.clone(),
                dest: neighbour.to_string(),
                body: Body {
                    id: None,
                    in_reply_to: None,
                    payload: GCounterPayload::Gossip { messages },
                },
            };
            send(&message, output)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_loop::<_, GCounterNode>().await
}
