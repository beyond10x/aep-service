//! Transactional PostgreSQL persistence, indexes and definition-bundle storage.
//!
//! No in-process hydrated copy is authoritative. Commands will read the revisions they decide on
//! and commit every resulting record within one database transaction.
