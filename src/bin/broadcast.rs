use gossip_glomers::*;
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast { message: usize },
    BroadcastOk,
    Read,
    // HashSet serializes to a list in JSON
    ReadOk { messages: HashSet<usize> },
    Topology { topology: Topology },
    TopologyOk,
    Gossip { messages: Vec<usize> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Topology {
    #[serde(flatten)]
    neighbours: HashMap<String, Vec<String>>,
}

struct BroadcastNode {
    id: usize,
    node_id: String,
    messages: HashSet<usize>,
    known_messages: HashMap<String, HashSet<usize>>,
    neighbours: Vec<String>,
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            node_id: init.node_id,
            messages: HashSet::new(),
            known_messages: HashMap::from_iter(
                init.node_ids
                    .clone()
                    .into_iter()
                    .map(|n| (n, HashSet::new())),
            ),
            neighbours: init.node_ids,
        }
    }

    fn handle(
        &mut self,
        input: Message<BroadcastPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(self.id));
        match reply.body.payload {
            BroadcastPayload::Broadcast { message } => {
                // Store in list of messages
                self.messages.insert(message);
                reply.body.payload = BroadcastPayload::BroadcastOk;
                send(&reply, output)?;
                self.id += 1;
            }
            BroadcastPayload::Read => {
                reply.body.payload = BroadcastPayload::ReadOk {
                    messages: self.messages.clone(),
                };

                send(&reply, output)?;
                self.id += 1;
            }
            BroadcastPayload::Topology { topology } => {
                if let Some(neighbours) = topology.neighbours.get(&self.node_id) {
                    self.neighbours = neighbours.to_vec();
                }
                reply.body.payload = BroadcastPayload::TopologyOk;
                send(&reply, output)?;
                self.id += 1;
            }
            BroadcastPayload::Gossip { messages } => {
                self.messages.extend(messages);
            }
            BroadcastPayload::TopologyOk
            | BroadcastPayload::ReadOk { .. }
            | BroadcastPayload::BroadcastOk { .. } => {}
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
                .filter(|m| !known.contains(m))
                .collect();
            let message = Message {
                src: self.node_id.clone(),
                dest: neighbour.to_string(),
                body: Body {
                    id: None,
                    in_reply_to: None,
                    payload: BroadcastPayload::Gossip { messages },
                },
            };
            send(&message, output)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_loop::<_, BroadcastNode>().await
}
