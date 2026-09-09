use crate::{OpensearchClient, Result, delete, upsert};

pub use upsert::agent_session::{AgentSessionMessageDocument, ReconcileAgentSessionArgs};

impl OpensearchClient {
    /// Replace the searchable projection of one agent session with an
    /// authoritative folded snapshot.
    #[tracing::instrument(skip(self, args), err)]
    pub async fn reconcile_agent_session(
        &self,
        args: &ReconcileAgentSessionArgs,
        index_override: Option<&str>,
    ) -> Result<()> {
        upsert::agent_session::reconcile_agent_session(&self.inner, args, index_override).await
    }

    /// Delete the parent and every folded-message child for an agent session.
    #[tracing::instrument(skip(self), err)]
    pub async fn delete_agent_session(
        &self,
        agent_session_id: &str,
        index_override: Option<&str>,
    ) -> Result<()> {
        delete::agent_session::delete_agent_session(&self.inner, agent_session_id, index_override)
            .await
    }
}
