//! Dialog windows for `RustConn`

mod connection;
mod import;
mod password;
mod progress;
mod settings;
mod snippet;

pub use connection::ConnectionDialog;
pub use import::ImportDialog;
pub use password::{PasswordDialog, PasswordDialogResult};
pub use progress::ProgressDialog;
pub use settings::SettingsDialog;
pub use snippet::SnippetDialog;

use rustconn_core::import::ImportResult;
use rustconn_core::models::{Connection, Snippet};
use rustconn_core::config::AppSettings;
use std::cell::RefCell;
use std::rc::Rc;

/// Type alias for connection dialog callback
pub type ConnectionCallback = Rc<RefCell<Option<Box<dyn Fn(Option<Connection>)>>>>;

/// Type alias for import dialog callback
pub type ImportCallback = Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>)>>>>;

/// Type alias for import dialog callback with source name
pub type ImportWithSourceCallback = Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>, String)>>>>;

/// Type alias for settings dialog callback
pub type SettingsCallback = Rc<RefCell<Option<Box<dyn Fn(Option<AppSettings>)>>>>;

/// Type alias for snippet dialog callback
pub type SnippetCallback = Rc<RefCell<Option<Box<dyn Fn(Option<Snippet>)>>>>;
