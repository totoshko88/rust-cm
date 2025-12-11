//! Import engine for migrating connections from other tools.
//!
//! This module provides functionality to import connections from various sources:
//! - SSH config files (~/.ssh/config)
//! - Asbru-CM configuration
//! - Remmina connection files
//! - Ansible inventory files

mod ansible;
mod asbru;
mod remmina;
mod ssh_config;
mod traits;

pub use ansible::AnsibleInventoryImporter;
pub use asbru::AsbruImporter;
pub use remmina::RemminaImporter;
pub use ssh_config::SshConfigImporter;
pub use traits::{ImportResult, ImportSource, SkippedEntry};
