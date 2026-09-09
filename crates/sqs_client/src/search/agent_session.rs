#[cfg(test)]
mod test;

/// SQS work item for reconciling an agent session from its authoritative log.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct AgentSession {
    /// Session whose folded projection should be rebuilt.
    pub agent_session_id: String,
    /// Optional physical OpenSearch index used by blue/green backfills.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub index_override: Option<String>,
}
