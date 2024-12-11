use gossip_glomers::*;
use std::collections::{HashMap, HashSet};

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
    ) -> anyhow::Result<Option<Message<BroadcastPayload>>> {
        let mut reply = input.into_reply(Some(self.id));
        let message = match reply.body.payload {
            BroadcastPayload::Broadcast { message } => {
                // Store in list of messages
                self.messages.insert(message);
                self.id += 1;
                reply.body.payload = BroadcastPayload::BroadcastOk;
                Some(reply)
            }
            BroadcastPayload::Read => {
                reply.body.payload = BroadcastPayload::ReadOk {
                    messages: self.messages.clone(),
                };
                self.id += 1;
                Some(reply)
            }
            BroadcastPayload::Topology { topology } => {
                if let Some(neighbours) = topology.neighbours.get(&self.node_id) {
                    self.neighbours = neighbours.to_vec();
                }
                reply.body.payload = BroadcastPayload::TopologyOk;
                self.id += 1;
                Some(reply)
            }
            BroadcastPayload::Gossip { messages } => {
                self.messages.extend(messages);
                None
            }
            BroadcastPayload::TopologyOk
            | BroadcastPayload::ReadOk { .. }
            | BroadcastPayload::BroadcastOk { .. } => None,
        };
        Ok(message)
    }

    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<BroadcastPayload>>> {
        let mut gossips = Vec::new();
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
            gossips.push(message);
        }
        Ok(gossips)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_loop::<_, BroadcastNode>().await
}
