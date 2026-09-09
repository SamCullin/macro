use embedding::{Content, RerankModel, Reranked, SearchResults};

use super::{cohere::CohereReranker, cosine::CosineReranker};

/// The production task-dedup reranker selected from the available provider
/// configuration.
#[derive(Clone)]
pub enum TaskDedupReranker {
    /// Cohere's cross-encoder reranker.
    Cohere(CohereReranker),
    /// The existing vector score, used when Cohere is not configured.
    Cosine(CosineReranker),
}

impl TaskDedupReranker {
    /// Selects Cohere when a provider-owned key is present, otherwise uses the
    /// deliberate vector-similarity fallback.
    pub fn from_cohere_api_key(api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        if is_configured_key(&api_key) {
            Self::Cohere(CohereReranker::new(api_key))
        } else {
            tracing::warn!(
                "COHERE_API_KEY is not configured; task dedup will use vector-similarity reranking"
            );
            Self::Cosine(CosineReranker)
        }
    }
}

fn is_configured_key(api_key: &str) -> bool {
    let normalised = api_key.trim().to_ascii_lowercase();
    !normalised.is_empty()
        && !normalised.starts_with("local-")
        && !["placeholder", "changeme", "replace-me", "example"]
            .iter()
            .all(|marker| !normalised.contains(marker))
}

impl<const DIMS: usize> RerankModel<DIMS> for TaskDedupReranker {
    async fn rerank<'a, T: Send>(
        &self,
        query: Content<'a>,
        candidates: Vec<SearchResults<T, DIMS>>,
    ) -> anyhow::Result<Vec<Reranked<T>>> {
        match self {
            Self::Cohere(reranker) => reranker.rerank(query, candidates).await,
            Self::Cosine(reranker) => reranker.rerank(query, candidates).await,
        }
    }
}
