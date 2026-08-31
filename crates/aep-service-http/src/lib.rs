//! HTTP realization of the EP-owned command/query service contract.
//!
//! This adapter will extract credentials and transport metadata, then hand a verified request to
//! the application crate. It owns no AEP decision semantics.
