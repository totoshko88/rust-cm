//! Connection management module
//!
//! This module provides the `ConnectionManager` for CRUD operations on connections
//! and groups, with persistence through `ConfigManager`.

mod manager;

pub use manager::ConnectionManager;
