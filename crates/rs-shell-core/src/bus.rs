use crate::events::ShellEvent;
use futures::Stream;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

const BUS_CAPACITY: usize = 512;

/// Global event bus. Clone the sender to publish; subscribe for a receiver.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<ShellEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self { tx }
    }

    /// Publish an event. Returns the number of active receivers (0 means no subscribers).
    pub fn publish(&self, event: ShellEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }

    /// Subscribe to all events.
    pub fn subscribe(&self) -> broadcast::Receiver<ShellEvent> {
        self.tx.subscribe()
    }

    /// Subscribe and filter by a predicate. Returns a filtered stream.
    pub fn subscribe_filtered<F>(&self, filter: F) -> impl Stream<Item = ShellEvent>
    where
        F: Fn(&ShellEvent) -> bool + Send + 'static,
    {
        let mut rx = self.subscribe();
        let (tx, out_rx) = mpsc::channel(BUS_CAPACITY);
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if filter(&event) && tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        ReceiverStream::new(out_rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{BatteryEvent, BatteryState, BatteryStatus, ShellEvent};
    use futures::StreamExt;

    #[tokio::test]
    async fn publish_subscribe_round_trip() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        let state = BatteryState {
            percentage: 42.0,
            status: BatteryStatus::Discharging,
            time_to_empty_secs: None,
            time_to_full_secs: None,
            voltage: None,
            temperature: None,
        };
        let event = ShellEvent::Battery(BatteryEvent::StateChanged(state));
        bus.publish(event.clone());
        let received = rx.recv().await.unwrap();
        assert!(matches!(received, ShellEvent::Battery(BatteryEvent::StateChanged(_))));
    }

    #[tokio::test]
    async fn filtered_subscription() {
        let bus = EventBus::new();
        let mut stream = bus.subscribe_filtered(|e| matches!(e, ShellEvent::Shell(_)));
        bus.publish(ShellEvent::Battery(BatteryEvent::PluggedIn));
        bus.publish(ShellEvent::Shell(crate::events::ShellLifecycleEvent::DaemonStarted));
        let received = stream.next().await.unwrap();
        assert!(matches!(received, ShellEvent::Shell(_)));
    }
}
