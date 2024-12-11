use gossip_glomers::*;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum KafkaPayload {
    Send {
        key: String,
        msg: usize,
    },
    SendOk {
        offset: usize,
    },
    Poll {
        offsets: HashMap<String, usize>,
    },
    PollOk {
        msgs: HashMap<String, Vec<(usize, usize)>>,
    },
    CommitOffsets {
        offsets: HashMap<String, usize>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>,
    },
}

#[derive(Default, Clone)]
struct LogMessage {
    offset: usize,
    value: usize,
}

#[allow(unused)]
struct KafkaNode {
    id: usize,
    node_id: String,
    logs: HashMap<String, Vec<LogMessage>>,
    committed: HashMap<String, usize>,
    latest_offset: HashMap<String, usize>,
}

impl KafkaNode {
    fn get_poll_msgs(
        &self,
        offsets: HashMap<String, usize>,
    ) -> HashMap<String, Vec<(usize, usize)>> {
        offsets
            .into_iter()
            .map(|(key, offset)| {
                let log = self.logs.get(&key).cloned().unwrap_or_default();
                let msgs: Vec<(usize, usize)> = log
                    .into_iter()
                    .filter(|msg| msg.offset >= offset)
                    .map(|msg| (msg.offset, msg.value))
                    .collect();
                (key, msgs)
            })
            .collect()
    }
}

impl Node<KafkaPayload> for KafkaNode {
    fn new(id: usize, init: Init) -> Self {
        Self {
            id,
            node_id: init.node_id,
            logs: HashMap::new(),
            committed: HashMap::new(),
            latest_offset: HashMap::new(),
        }
    }

    fn handle(
        &mut self,
        input: Message<KafkaPayload>,
    ) -> anyhow::Result<Option<Message<KafkaPayload>>> {
        let mut reply = input.into_reply(Some(self.id));
        let message = match reply.body.payload {
            KafkaPayload::Send { key, msg } => {
                let logs = self.logs.entry(key.clone()).or_default();
                let last_offset = self.latest_offset.entry(key.clone()).or_default();
                *last_offset += 1;
                let offset = *last_offset;
                logs.push(LogMessage { offset, value: msg });
                logs.sort_by_key(|msg| msg.offset);

                reply.body.payload = KafkaPayload::SendOk { offset };
                Some(reply)
            }
            KafkaPayload::Poll { offsets } => {
                reply.body.payload = KafkaPayload::PollOk {
                    msgs: self.get_poll_msgs(offsets),
                };
                Some(reply)
            }
            KafkaPayload::CommitOffsets { offsets } => {
                for (key, offset) in offsets {
                    self.committed.insert(key, offset);
                }
                reply.body.payload = KafkaPayload::CommitOffsetsOk;
                Some(reply)
            }
            KafkaPayload::ListCommittedOffsets { keys } => {
                let offsets: HashMap<_, _> = keys
                    .into_iter()
                    .map(|key| {
                        (
                            key.clone(),
                            self.committed.get(&key).cloned().unwrap_or_default(),
                        )
                    })
                    .collect();
                reply.body.payload = KafkaPayload::ListCommittedOffsetsOk { offsets };
                Some(reply)
            }
            KafkaPayload::SendOk { .. }
            | KafkaPayload::PollOk { .. }
            | KafkaPayload::CommitOffsetsOk
            | KafkaPayload::ListCommittedOffsetsOk { .. } => None,
        };
        Ok(message)
    }

    fn handle_gossip(&self) -> anyhow::Result<Vec<Message<KafkaPayload>>> {
        Ok(vec![])
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_loop::<_, KafkaNode>().await
}
