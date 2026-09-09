//! Postgres source and OpenSearch sink for agent-session indexing.

use std::sync::Arc;

use agent_session::{
    domain::{
        model::AgentSessionId,
        ports::{AgentSessionLogRepo, AgentSessionRepo},
    },
    outbound::postgres::PgAgentSessionRepo,
};
use opensearch_client::{
    OpensearchClient,
    agent_session::{AgentSessionMessageDocument, ReconcileAgentSessionArgs},
    date_format::EpochMillis,
};
use rootcause::Report;
use sqlx::PgPool;

use crate::domain::agent_session_index::{
    AgentSessionIndexService, AgentSessionSearchIndex, AgentSessionSearchProjection,
    AgentSessionSearchSnapshot, AgentSessionSearchSource,
};

/// Concrete agent-session indexer assembled by the service composition root.
pub type AgentSessionIndexer =
    AgentSessionIndexService<PgAgentSessionSearchSource, OpenSearchAgentSessionIndex>;

/// Reads authoritative session rows and effective ACP history from Postgres.
#[derive(Debug, Clone)]
pub struct PgAgentSessionSearchSource {
    pool: PgPool,
    repo: PgAgentSessionRepo,
}

impl PgAgentSessionSearchSource {
    /// Construct a source over the primary MacroDB pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: PgAgentSessionRepo::new(pool.clone()),
            pool,
        }
    }
}

impl AgentSessionSearchSource for PgAgentSessionSearchSource {
    async fn load(&self, id: AgentSessionId) -> Result<Option<AgentSessionSearchSnapshot>, Report> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM agent_session WHERE id = $1) AS \"exists!\"",
            id.as_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| rootcause::report!(error))?;
        if !exists {
            return Ok(None);
        }

        let session = self
            .repo
            .get(id)
            .await
            .map_err(|error| rootcause::report!(error))?;
        let stored = AgentSessionLogRepo::list_by_session(&self.repo, id)
            .await
            .map_err(|error| rootcause::report!(error))?;
        let last_log_id = stored.last().map(|entry| entry.id);
        let log = stored.into_iter().map(|entry| entry.entry).collect();

        Ok(Some(AgentSessionSearchSnapshot {
            id: session.id,
            name: session.name,
            owner_id: session.owner_id,
            bot_id: session.bot_id.as_uuid(),
            thread_id: session.thread_id,
            originating_message_id: session.originating_message_id,
            created_at: session.created_at,
            modified_at: session.modified_at,
            log,
            last_log_id,
        }))
    }
}

/// Writes folded agent-session projections to OpenSearch.
#[derive(Debug, Clone)]
pub struct OpenSearchAgentSessionIndex {
    client: Arc<OpensearchClient>,
}

impl OpenSearchAgentSessionIndex {
    /// Construct an index adapter.
    pub fn new(client: Arc<OpensearchClient>) -> Self {
        Self { client }
    }
}

impl AgentSessionSearchIndex for OpenSearchAgentSessionIndex {
    async fn reconcile(
        &self,
        projection: AgentSessionSearchProjection,
        index_override: Option<&str>,
    ) -> Result<(), Report> {
        let created_at_millis = EpochMillis::new(projection.created_at.timestamp_millis())
            .map_err(|error| rootcause::report!(error))?;
        let updated_at_millis = EpochMillis::new(projection.modified_at.timestamp_millis())
            .map_err(|error| rootcause::report!(error))?;
        let args = ReconcileAgentSessionArgs {
            agent_session_id: projection.id.to_string(),
            name: projection.name,
            owner_id: projection.owner_id.to_string(),
            bot_id: projection.bot_id.to_string(),
            thread_id: projection.thread_id.map(|id| id.to_string()),
            originating_message_id: projection.originating_message_id.map(|id| id.to_string()),
            created_at_millis,
            updated_at_millis,
            projection_generation: projection.generation,
            messages: projection
                .messages
                .into_iter()
                .map(|message| AgentSessionMessageDocument {
                    turn: message.turn,
                    author: message.author.as_str().to_owned(),
                    author_user_id: message.author_user_id.map(|id| id.to_string()),
                    content: message.content,
                })
                .collect(),
        };
        self.client
            .reconcile_agent_session(&args, index_override)
            .await
            .map_err(|error| -> Report { rootcause::report!(error).into() })
    }

    async fn delete(&self, id: AgentSessionId, index_override: Option<&str>) -> Result<(), Report> {
        self.client
            .delete_agent_session(&id.to_string(), index_override)
            .await
            .map_err(|error| -> Report { rootcause::report!(error).into() })
    }
}
