use tokio::sync::mpsc;

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
