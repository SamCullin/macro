//! Application service for rebuilding the searchable view of an agent session.
//!
//! The source port returns the authoritative persisted session and ACP log;
//! this domain service applies `agent_fold`, then asks the index port to store
//! only the resulting user-visible projection.

use std::future::Future;

use agent_fold::domain::{
    fold::fold,
    log::{AgentSessionId, AgentSessionLog},
    model::{Author, AuthorKind},
};
use chrono::{DateTime, Utc};
use macro_user_id::user_id::MacroUserIdStr;
use macro_uuid::Uuid;
use rootcause::Report;

#[cfg(test)]
mod test;

/// Bump whenever folding-to-search semantics change so a rebuild can prune
/// documents emitted by an older projection algorithm.
const PROJECTION_VERSION: &str = "v1";

/// Persisted source facts needed to derive one session's search documents.
#[derive(Debug, Clone)]
pub struct AgentSessionSearchSnapshot {
    /// Session identity.
    pub id: AgentSessionId,
    /// User-facing name.
    pub name: String,
    /// Owner from the authoritative session row.
    pub owner_id: MacroUserIdStr<'static>,
    /// Bot persona behind the session.
    pub bot_id: Uuid,
    /// Originating thread, when any.
    pub thread_id: Option<Uuid>,
    /// Message that opened the session, when any.
    pub originating_message_id: Option<Uuid>,
    /// Session creation time.
    pub created_at: DateTime<Utc>,
    /// Latest persisted session metadata time.
    pub modified_at: DateTime<Utc>,
    /// Effective ACP history in deterministic order.
    pub log: Vec<AgentSessionLog>,
    /// Last durable log row in the effective history, when one exists.
    pub last_log_id: Option<Uuid>,
}

/// One child document derived from a folded message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionMessageProjection {
    /// Fold-assigned turn number.
    pub turn: u32,
    /// Fold-assigned author side.
    pub author: AuthorKind,
    /// Attributed prompt author, when known.
    pub author_user_id: Option<MacroUserIdStr<'static>>,
    /// Searchable text flattened from the renderable message.
    pub content: String,
}

/// Complete parent/child projection written in one reconciliation.
#[derive(Debug, Clone)]
pub struct AgentSessionSearchProjection {
    /// Session identity.
    pub id: AgentSessionId,
    /// User-facing name.
    pub name: String,
    /// Session owner.
    pub owner_id: MacroUserIdStr<'static>,
    /// Bot persona.
    pub bot_id: Uuid,
    /// Originating thread.
    pub thread_id: Option<Uuid>,
    /// Originating message.
    pub originating_message_id: Option<Uuid>,
    /// Session creation time.
    pub created_at: DateTime<Utc>,
    /// Session metadata modification time.
    pub modified_at: DateTime<Utc>,
    /// Deterministic generation of the source snapshot.
    pub generation: String,
    /// Current folded messages.
    pub messages: Vec<AgentSessionMessageProjection>,
}

/// Authoritative source of an agent session and its effective ACP history.
pub trait AgentSessionSearchSource: Send + Sync + 'static {
    /// Load a session snapshot. `None` means the session no longer exists.
    fn load(
        &self,
        id: AgentSessionId,
    ) -> impl Future<Output = Result<Option<AgentSessionSearchSnapshot>, Report>> + Send;
}

/// Search-store operations required by the indexing use case.
pub trait AgentSessionSearchIndex: Send + Sync + 'static {
    /// Replace one session's parent and children with this projection.
    fn reconcile(
        &self,
        projection: AgentSessionSearchProjection,
        index_override: Option<&str>,
    ) -> impl Future<Output = Result<(), Report>> + Send;

    /// Remove every search document belonging to one session.
    fn delete(
        &self,
        id: AgentSessionId,
        index_override: Option<&str>,
    ) -> impl Future<Output = Result<(), Report>> + Send;
}

/// Rebuilds agent-session search documents from the persisted source of truth.
#[derive(Debug, Clone)]
pub struct AgentSessionIndexService<Source, Index> {
    source: Source,
    index: Index,
}

impl<Source, Index> AgentSessionIndexService<Source, Index> {
    /// Construct the indexing application service from its two outbound ports.
    pub fn new(source: Source, index: Index) -> Self {
        Self { source, index }
    }
}

impl<Source, Index> AgentSessionIndexService<Source, Index>
where
    Source: AgentSessionSearchSource,
    Index: AgentSessionSearchIndex,
{
    /// Re-read and completely reconcile a session's folded projection.
    #[tracing::instrument(err, skip(self, index_override))]
    pub async fn reconcile(
        &self,
        id: AgentSessionId,
        index_override: Option<&str>,
    ) -> Result<(), Report> {
        let Some(snapshot) = self.source.load(id).await? else {
            return self.index.delete(id, index_override).await;
        };

        self.index
            .reconcile(project_snapshot(snapshot), index_override)
            .await
    }

    /// Remove a deleted session without attempting to read its vanished log.
    #[tracing::instrument(err, skip(self, index_override))]
    pub async fn delete(
        &self,
        id: AgentSessionId,
        index_override: Option<&str>,
    ) -> Result<(), Report> {
        self.index.delete(id, index_override).await
    }
}

fn project_snapshot(snapshot: AgentSessionSearchSnapshot) -> AgentSessionSearchProjection {
    let generation = format!(
        "{PROJECTION_VERSION}:{}:{}",
        snapshot.modified_at.timestamp_micros(),
        snapshot
            .last_log_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "empty".to_owned())
    );
    let messages = fold(snapshot.log)
        .into_iter()
        .map(|message| {
            let author_user_id = match &message.author {
                Author::User { user_id } => user_id.clone(),
                Author::Agent => None,
            };
            AgentSessionMessageProjection {
                turn: message.id.0,
                author: message.author.kind(),
                author_user_id,
                content: message.searchable_text(),
            }
        })
        .collect();

    AgentSessionSearchProjection {
        id: snapshot.id,
        name: snapshot.name,
        owner_id: snapshot.owner_id,
        bot_id: snapshot.bot_id,
        thread_id: snapshot.thread_id,
        originating_message_id: snapshot.originating_message_id,
        created_at: snapshot.created_at,
        modified_at: snapshot.modified_at,
        generation,
        messages,
    }
}
