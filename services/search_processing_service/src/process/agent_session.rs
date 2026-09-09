//! Queue processing for authoritative agent-session reconciliation.

use agent_fold::domain::log::AgentSessionId;
use anyhow::Context as _;
use macro_uuid::Uuid;

use crate::outbound::agent_session_search::AgentSessionIndexer;

/// Parse a work item and hand it to the indexing application service.
pub(crate) async fn reconcile_agent_session(
    indexer: &AgentSessionIndexer,
    agent_session_id: &str,
    index_override: Option<&str>,
) -> anyhow::Result<()> {
    let id = AgentSessionId::new_from_uuid(
        Uuid::parse_str(agent_session_id).context("invalid agent session id")?,
    );
    indexer
        .reconcile(id, index_override)
        .await
        .map_err(|error| anyhow::anyhow!("failed to reconcile agent session: {error:?}"))
}
