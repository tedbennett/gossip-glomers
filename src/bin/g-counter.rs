use gossip_glomers::*;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum GCounterPayload {
    Add {
        delta: usize,
    },
    AddOk,
    Read,
    ReadOk {
        value: usize,
    },
    Topology {
        topology: Topology,
    },
    TopologyOk,
    Gossip {
        #[serde(flatten)]
        values: HashMap<String, usize>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Topology {
    #[serde(flatten)]
    neighbours: HashMap<String, Vec<String>>,
}

struct GCounterNode {
    id: usize,
    node_id: String,
    // Map of node_id -> count
    state: HashMap<String, usize>,
}

impl GCounterNode {
    fn add(&mut self, delta: usize) {
        *self.state.entry(self.node_id.clone()).or_default() += delta;
    }
}

impl Node<GCounterPayload> for GCounterNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            node_id: init.node_id,
            state: HashMap::from_iter(init.node_ids.into_iter().map(|n| (n, 0))),
        }
    }

    fn handle(
        &mut self,
        input: Message<GCounterPayload>,
    ) -> anyhow::Result<Option<Message<GCounterPayload>>> {
        let mut reply = input.into_reply(Some(self.id));
        let message = match reply.body.payload {
            GCounterPayload::Add { delta } => {
                // Store in total count
                self.add(delta);
                reply.body.payload = GCounterPayload::AddOk;
                self.id += 1;
                Some(reply)
            }
            GCounterPayload::Read => {
                reply.body.payload = GCounterPayload::ReadOk {
                    value: self.state.values().into_iter().sum(),
                };
                self.id += 1;
                Some(reply)
            }
            GCounterPayload::Topology { .. } => {
                // Topology is not used in this example
                // if let Some(neighbours) = topology.neighbours.get(&self.node_id) {
                //     self.neighbours = neighbours
                //         .to_vec()
                //         .into_iter()
                //         .filter(|n| n != &self.node_id)
                //         .collect();
                // }
                reply.body.payload = GCounterPayload::TopologyOk;
                self.id += 1;
                Some(reply)
            }
            GCounterPayload::Gossip { values } => {
                for (key, value) in values {
                    if key != self.node_id {
                        let count = self.state.get(&key).cloned().unwrap_or(0 as usize);
                        if count < value {
                            self.state.insert(key, value);
                        }
                    }
                }
                None
            }
            GCounterPayload::TopologyOk
            | GCounterPayload::ReadOk { .. }
            | GCounterPayload::AddOk { .. } => None,
        };
        Ok(message)
    }

    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<GCounterPayload>>> {
        let mut messages = Vec::new();
        for neighbour in self.state.keys() {
            if neighbour != &self.node_id {
                let message = Message {
                    src: self.node_id.clone(),
                    dest: neighbour.to_string(),
                    body: Body {
                        id: None,
                        in_reply_to: None,
                        payload: GCounterPayload::Gossip {
                            // Very rudimentary - just dump our whole database
                            // Probably would want to do a subset of counts we know
                            // If we were concerned with message size
                            values: self.state.clone(),
                        },
                    },
                };
                messages.push(message);
            }
        }
        Ok(messages)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_loop::<_, GCounterNode>().await
}
