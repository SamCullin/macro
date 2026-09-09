use embedding::{Content, RerankModel, Reranked, SearchResults};

/// Reranks candidates by the vector similarity already calculated by the store.
///
/// This is the intentional local fallback when no provider-owned Cohere key is
/// configured. It keeps task-dedup retrieval operational while preserving the
/// existing similarity ordering; a configured Cohere key still enables the
/// higher-quality cross-encoder reranker.
#[derive(Clone, Copy, Debug, Default)]
pub struct CosineReranker;

impl<const DIMS: usize> RerankModel<DIMS> for CosineReranker {
    async fn rerank<'a, T: Send>(
        &self,
        _query: Content<'a>,
        candidates: Vec<SearchResults<T, DIMS>>,
    ) -> anyhow::Result<Vec<Reranked<T>>> {
        let mut ranked = candidates
            .into_iter()
            .map(|candidate| {
                let score = candidate
                    .matches
                    .first()
                    .map(|matched| matched.score)
                    .unwrap_or(0.0);
                Reranked {
                    item: candidate.metadata,
                    score,
                }
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| right.score.total_cmp(&left.score));
        Ok(ranked)
    }
}
