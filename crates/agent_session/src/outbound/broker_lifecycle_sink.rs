//! Publishing session lifecycle events through the macro event broker.

use macro_event_broker::MacroEventBroker;

use crate::domain::events::AgentSessionLifecycleMacroEvent;
use crate::domain::ports::AgentSessionLifecycleSink;

/// An [`AgentSessionLifecycleSink`] that hands each event to a
/// [`MacroEventBroker`], which serializes it and publishes it to the
/// lifecycle topic on a spawned task.
///
/// `send_event` only fails synchronously on serialization, and publication
/// failures are logged by the broker's task, so the sink's contract - never
/// fail the mutation - is met by logging the former and dropping the handle.
#[derive(Debug, Clone)]
pub struct BrokerLifecycleSink<B> {
    broker: B,
}

impl<B: MacroEventBroker> BrokerLifecycleSink<B> {
    /// Wrap `broker`.
    pub fn new(broker: B) -> Self {
        Self { broker }
    }
}

impl<B: MacroEventBroker> AgentSessionLifecycleSink for BrokerLifecycleSink<B> {
    fn publish(&self, event: AgentSessionLifecycleMacroEvent) {
        drop(self.broker.send_event(&event).inspect_err(|error| {
            tracing::error!(error = ?error, "failed to schedule agent session lifecycle event");
        }));
    }
}
