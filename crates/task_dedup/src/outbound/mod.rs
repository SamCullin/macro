//! Outbound adapters for task duplicate detection.

pub mod cohere;
pub mod connection_gateway;
/// Vector-similarity fallback for reranking.
pub mod cosine;
pub mod judge;
pub mod postgres;
/// Provider-selected task-dedup reranker.
pub mod reranker;
