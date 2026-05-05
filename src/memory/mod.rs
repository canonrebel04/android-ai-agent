//! Memory subsystem — holographic fact store, entity indexing, context injection.
//!
//! Architecture:
//!   fact_store.rs  — SQLite + FTS5 database (add/search/probe/reason/contradict/feedback)
//!   fact_index.rs  — Entity extraction, full probe pipeline, cross-entity reasoning
//!   injector.rs    — Assemble facts + web results into compact XML for prompt injection

pub mod chat_store;
pub mod compactor;
pub mod deduplicator;
pub mod fact_index;
pub mod fact_store;
pub mod injector;
pub mod pre_fetcher;
