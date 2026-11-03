//! Stage 6 preview: a bounded async channel wrapped in a small type.

use tokio::sync::mpsc;

/// The sending half of a bounded queue. When the buffer is full, `send`
/// waits instead of growing memory without limit (backpressure).
#[derive(Debug, Clone)]
pub struct MessageQueue {
    sender: mpsc::Sender<String>,
}

impl MessageQueue {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<String>) {
        let (sender, receiver) = mpsc::channel(buffer);
        (Self { sender }, receiver)
    }

    pub async fn send(&self, msg: &str) -> Result<(), mpsc::error::SendError<String>> {
        self.sender.send(msg.to_string()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn messages_arrive_in_order() {
        let (queue, mut rx) = MessageQueue::new(4);
        for msg in ["a", "b", "c"] {
            queue.send(msg).await.unwrap();
        }
        drop(queue); // last sender gone: the receiver sees the end of the stream
        let mut got = Vec::new();
        while let Some(msg) = rx.recv().await {
            got.push(msg);
        }
        assert_eq!(got, ["a", "b", "c"]);
    }

    #[tokio::test]
    async fn send_fails_after_receiver_is_dropped() {
        let (queue, rx) = MessageQueue::new(1);
        drop(rx);
        assert!(queue.send("lost").await.is_err());
    }
}
