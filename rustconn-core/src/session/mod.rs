//! Session management for RustConn
//!
//! This module provides session lifecycle management for active connections,
//! including process handling, logging, and terminal integration.

mod manager;
mod session;
mod logger;

pub use manager::SessionManager;
pub use session::{Session, SessionState, SessionType};
pub use logger::SessionLogger;
