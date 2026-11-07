use super::events::DownloadEvent;
use tokio::sync::broadcast;

/// Central event broadcaster decoupling engine from UI / telemetry consumers.
#[derive(Clone)]
pub struct EventDispatcher {
    sender: broadcast::Sender<DownloadEvent>,
}

impl EventDispatcher {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Dispatch event to all active subscribers.
    pub fn emit(&self, event: DownloadEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new(512)
    }
}
