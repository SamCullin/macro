//! Kafka adapter for agent-session search invalidations.

use std::pin::Pin;

use agent_session_events::AgentSessionSearchMacroEvent;
use macro_event_broker::MacroEventBroker;

use crate::domain::{
    model::AgentSessionId,
    ports::{AgentSessionSearchEvents, SessionTurnObserver},
};

/// Publishes search invalidations through Macro's typed event broker.
#[derive(Clone)]
pub struct BrokerAgentSessionSearchEvents<Broker> {
    broker: Broker,
}

impl<Broker> BrokerAgentSessionSearchEvents<Broker> {
    /// Construct the adapter around an event broker.
    pub fn new(broker: Broker) -> Self {
        Self { broker }
    }

    fn enqueue(&self, event: AgentSessionSearchMacroEvent) -> Result<(), rootcause::Report>
    where
        Broker: MacroEventBroker,
    {
        self.broker
            .send_event(&event)
            .map(|_| ())
            .map_err(|error| -> rootcause::Report { rootcause::report!(error).into() })
    }
}

impl<Broker> AgentSessionSearchEvents for BrokerAgentSessionSearchEvents<Broker>
where
    Broker: MacroEventBroker,
{
    fn reconcile(
        &self,
        id: AgentSessionId,
    ) -> Pin<Box<dyn Future<Output = Result<(), rootcause::Report>> + Send + '_>> {
        Box::pin(async move { self.enqueue(AgentSessionSearchMacroEvent::reconcile(id.as_uuid())) })
    }

    fn deleted(
        &self,
        id: AgentSessionId,
    ) -> Pin<Box<dyn Future<Output = Result<(), rootcause::Report>> + Send + '_>> {
        Box::pin(async move { self.enqueue(AgentSessionSearchMacroEvent::deleted(id.as_uuid())) })
    }
}

/// Turn completion and disconnect are stable points at which to rebuild the
/// transcript. Publication is handed to the broker synchronously; the broker
/// task performs the I/O without blocking the actor.
impl<Broker> SessionTurnObserver for BrokerAgentSessionSearchEvents<Broker>
where
    Broker: MacroEventBroker,
{
    fn turn_ended(&self, id: AgentSessionId) {
        self.enqueue(AgentSessionSearchMacroEvent::reconcile(id.as_uuid())).inspect_err(|error| {
            tracing::error!(error=?error, %id, "failed to enqueue agent-session search reconcile");
        }).ok();
    }

    fn session_stopped(&self, id: AgentSessionId) {
        self.enqueue(AgentSessionSearchMacroEvent::reconcile(id.as_uuid())).inspect_err(|error| {
            tracing::error!(error=?error, %id, "failed to enqueue stopped agent-session search reconcile");
        }).ok();
    }
}
