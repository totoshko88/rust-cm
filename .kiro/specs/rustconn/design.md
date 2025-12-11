# Design Document: RustConn

## Overview

RustConn is a modern connection manager for Linux written in Rust, designed to replace legacy tools like Asbru-CM with a native Wayland-first approach. The application provides a unified interface for managing SSH, RDP, and VNC connections with secure credential storage and extensive import capabilities.

### Key Design Goals

- **Native Wayland**: No X11 dependencies, full portal integration
- **Security First**: KeePassXC integration with libsecret fallback
- **Extensibility**: Protocol-agnostic architecture via traits
- **Performance**: Async I/O with tokio, responsive GTK4 UI
- **Migration Path**: Import from Asbru-CM, Remmina, SSH config, Ansible

### Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 1.75+ |
| GUI Framework | GTK4 4.12+ (gtk4-rs) |
| Terminal | VTE 0.74+ GTK4 variant (vte4) |
| Async Runtime | Tokio |
| Serialization | TOML (toml crate), YAML (serde_yaml) |
| Secret Storage | KeePassXC (keepassxc-browser protocol), libsecret |
| RDP Client | FreeRDP (xfreerdp) |
| VNC Clients | TightVNC, TigerVNC (vncviewer) |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        RustConn Application                      │
├─────────────────────────────────────────────────────────────────┤
│                           GUI Layer                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │MainWindow│ │Connection│ │ Settings │ │   Import Dialog  │   │
│  │          │ │  Dialog  │ │  Dialog  │ │                  │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Terminal Tabs (VTE4)                   │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                        Core Library                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │
│  │ Connection │ │  Protocol  │ │   Secret   │ │   Config   │   │
│  │  Manager   │ │  Registry  │ │  Manager   │ │  Manager   │   │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐                   │
│  │  Importer  │ │  Snippet   │ │  Session   │                   │
│  │   Engine   │ │  Manager   │ │  Logger    │                   │
│  └────────────┘ └────────────┘ └────────────┘                   │
├─────────────────────────────────────────────────────────────────┤
│                      Protocol Layer                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐                   │
│  │    SSH     │ │    RDP     │ │    VNC     │                   │
│  │  Protocol  │ │  Protocol  │ │  Protocol  │                   │
│  └────────────┘ └────────────┘ └────────────┘                   │
├─────────────────────────────────────────────────────────────────┤
│                      External Services                           │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │
│  │ KeePassXC  │ │ libsecret  │ │  FreeRDP   │ │ VNC Client │   │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

1. **GUI Layer**: GTK4 widgets, user interaction, terminal rendering
2. **Core Library**: Business logic, connection management, configuration
3. **Protocol Layer**: Protocol-specific implementations via trait objects
4. **External Services**: Integration with system services and external clients

## Components and Interfaces

### Protocol Trait

```rust
/// Core trait for all connection protocols
#[async_trait]
pub trait Protocol: Send + Sync {
    /// Returns the protocol identifier (e.g., "ssh", "rdp", "vnc")
    fn protocol_id(&self) -> &'static str;
    
    /// Returns human-readable protocol name
    fn display_name(&self) -> &'static str;
    
    /// Validates connection configuration
    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), ValidationError>;
    
    /// Builds command line arguments for the connection
    fn build_command(&self, config: &ConnectionConfig, credentials: Option<&Credentials>) -> Result<Command, ProtocolError>;
    
    /// Returns default port for this protocol
    fn default_port(&self) -> u16;
    
    /// Whether this protocol uses embedded terminal (SSH) or external window (RDP/VNC)
    fn uses_embedded_terminal(&self) -> bool;
    
    /// Protocol-specific configuration schema for UI generation
    fn config_schema(&self) -> ConfigSchema;
}
```

### Secret Manager Interface

```rust
/// Abstraction over secret storage backends
#[async_trait]
pub trait SecretBackend: Send + Sync {
    /// Store credentials for a connection
    async fn store(&self, connection_id: &str, credentials: &Credentials) -> Result<(), SecretError>;
    
    /// Retrieve credentials for a connection
    async fn retrieve(&self, connection_id: &str) -> Result<Option<Credentials>, SecretError>;
    
    /// Delete credentials for a connection
    async fn delete(&self, connection_id: &str) -> Result<(), SecretError>;
    
    /// Check if backend is available
    async fn is_available(&self) -> bool;
    
    /// Backend identifier
    fn backend_id(&self) -> &'static str;
}

/// Composite secret manager with fallback support
pub struct SecretManager {
    backends: Vec<Box<dyn SecretBackend>>,
}
```

### Connection Manager

```rust
pub struct ConnectionManager {
    connections: HashMap<Uuid, Connection>,
    groups: HashMap<Uuid, ConnectionGroup>,
    config_path: PathBuf,
}

impl ConnectionManager {
    pub fn create_connection(&mut self, config: ConnectionConfig) -> Result<Uuid, Error>;
    pub fn update_connection(&mut self, id: Uuid, config: ConnectionConfig) -> Result<(), Error>;
    pub fn delete_connection(&mut self, id: Uuid) -> Result<(), Error>;
    pub fn get_connection(&self, id: Uuid) -> Option<&Connection>;
    pub fn search(&self, query: &str) -> Vec<&Connection>;
    pub fn get_by_group(&self, group_id: Uuid) -> Vec<&Connection>;
    pub fn move_to_group(&mut self, connection_id: Uuid, group_id: Option<Uuid>) -> Result<(), Error>;
}
```

### Import Engine

```rust
/// Trait for import source implementations
pub trait ImportSource: Send + Sync {
    /// Source identifier
    fn source_id(&self) -> &'static str;
    
    /// Human-readable source name
    fn display_name(&self) -> &'static str;
    
    /// Check if source is available (files exist)
    fn is_available(&self) -> bool;
    
    /// Parse and return connections from source
    fn import(&self) -> Result<ImportResult, ImportError>;
}

pub struct ImportResult {
    pub connections: Vec<ConnectionConfig>,
    pub groups: Vec<ConnectionGroup>,
    pub skipped: Vec<SkippedEntry>,
    pub errors: Vec<ImportError>,
}

// Implementations
pub struct AsbruImporter { /* ... */ }
pub struct SshConfigImporter { /* ... */ }
pub struct RemminaImporter { /* ... */ }
pub struct AnsibleInventoryImporter { /* ... */ }
```

### Session Manager

```rust
pub struct SessionManager {
    active_sessions: HashMap<Uuid, Session>,
    process_manager: ProcessManager,
}

pub struct Session {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub terminal: Option<vte4::Terminal>,
    pub process: Child,
    pub log_file: Option<PathBuf>,
}

impl SessionManager {
    pub async fn start_session(&mut self, connection: &Connection, credentials: Option<Credentials>) -> Result<Uuid, SessionError>;
    pub async fn terminate_session(&mut self, session_id: Uuid) -> Result<(), SessionError>;
    pub fn get_active_sessions(&self) -> Vec<&Session>;
}
```

## Data Models

### Connection

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub name: String,
    pub protocol: ProtocolType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub group_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub protocol_config: ProtocolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolConfig {
    Ssh(SshConfig),
    Rdp(RdpConfig),
    Vnc(VncConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub auth_method: SshAuthMethod,
    pub key_path: Option<PathBuf>,
    pub proxy_jump: Option<String>,
    pub use_control_master: bool,
    pub custom_options: HashMap<String, String>,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshAuthMethod {
    Password,
    PublicKey,
    KeyboardInteractive,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConfig {
    pub client: RdpClient,
    pub resolution: Option<Resolution>,
    pub color_depth: Option<u8>,
    pub audio_redirect: bool,
    pub gateway: Option<RdpGateway>,
    pub custom_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RdpClient {
    FreeRdp,
    Custom(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VncConfig {
    pub client: VncClient,
    pub encoding: Option<String>,
    pub compression: Option<u8>,
    pub quality: Option<u8>,
    pub custom_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VncClient {
    TightVnc,
    TigerVnc,
    Custom(PathBuf),
}
```

### Connection Group

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionGroup {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub expanded: bool,
    pub created_at: DateTime<Utc>,
}
```

### Snippet

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub variables: Vec<SnippetVariable>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetVariable {
    pub name: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
}
```

### Credentials

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<SecretString>,
    pub key_passphrase: Option<SecretString>,
}
```

### Configuration Files Structure

```
~/.config/rustconn/
├── config.toml           # Application settings
├── connections.toml      # Connection definitions
├── groups.toml           # Group hierarchy
├── snippets.toml         # Command snippets
└── logs/                 # Session logs
    └── {connection_id}_{timestamp}.log
```

### TOML Schema Example

```toml
# connections.toml
[[connections]]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "Production Server"
protocol = "ssh"
host = "prod.example.com"
port = 22
username = "admin"
group_id = "550e8400-e29b-41d4-a716-446655440001"
tags = ["production", "web"]
created_at = "2024-01-15T10:30:00Z"
updated_at = "2024-01-15T10:30:00Z"

[connections.protocol_config]
type = "Ssh"
auth_method = "PublicKey"
key_path = "/home/user/.ssh/id_ed25519"
use_control_master = true

[connections.protocol_config.custom_options]
ServerAliveInterval = "60"
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the acceptance criteria analysis, the following correctness properties must be validated through property-based testing:

### Property 1: Connection CRUD Data Integrity

*For any* valid connection configuration, creating a connection and then retrieving it by ID should return a connection with identical name, host, port, protocol type, and all other configuration fields.

*For any* existing connection and valid update, updating the connection should preserve the original ID while changing only the specified fields.

*For any* existing connection, deleting it should result in the connection being absent from all queries and its group membership being removed.

**Validates: Requirements 1.1, 1.2, 1.3**

### Property 2: Connection Search Correctness

*For any* set of connections and search query, all returned results must match the query against at least one of: name, host, tags, or group path. No connection matching the query should be excluded from results.

**Validates: Requirements 1.5, 1.6**

### Property 3: SSH Command Builder Correctness

*For any* valid SSH connection configuration (including auth method, proxy jump, control master, and custom options), the built command must include all specified parameters in the correct SSH command-line format.

**Validates: Requirements 2.2, 2.3, 2.4, 2.5**

### Property 4: RDP Command Builder Correctness

*For any* valid RDP connection configuration (including resolution, color depth, audio redirect, gateway, and custom client), the built command must use the correct client binary and include all specified parameters in the correct format.

**Validates: Requirements 3.1, 3.2, 3.3, 3.5**

### Property 5: VNC Command Builder Correctness

*For any* valid VNC connection configuration (including client preference, encoding, compression, and quality), the built command must use the correct client binary and include all specified parameters.

**Validates: Requirements 4.1, 4.2, 4.3**

### Property 6: Connection Serialization Round-Trip

*For any* valid Connection object, serializing to TOML and then deserializing should produce an equivalent Connection object with all fields preserved.

**Validates: Requirements 10.5, 10.6**

### Property 7: SSH Config Import Parsing

*For any* valid SSH config file content, parsing should extract all Host entries with their corresponding parameters (hostname, port, user, identity file, proxy command) correctly mapped to Connection objects.

**Validates: Requirements 6.2, 6.3**

### Property 8: Asbru-CM Import Parsing

*For any* valid Asbru-CM configuration (YAML/XML), parsing should create Connection objects with correctly mapped protocol, host, port, and authentication settings.

**Validates: Requirements 6.1**

### Property 9: Remmina Import Parsing

*For any* valid .remmina file, parsing should create a Connection object with correctly mapped protocol type, server, port, and protocol-specific settings.

**Validates: Requirements 6.4**

### Property 10: Ansible Inventory Import Parsing

*For any* valid Ansible inventory (INI or YAML format), parsing should create Connection objects for each host with correctly mapped hostname, port, and ansible_user.

**Validates: Requirements 6.5**

### Property 11: Import Error Handling

*For any* import source containing a mix of valid and invalid entries, the import result should contain all valid entries as connections, all invalid entries in the skipped list, and the counts should match (successful + skipped + errors = total entries).

**Validates: Requirements 6.6, 6.7**

### Property 12: Snippet Variable Extraction

*For any* snippet command template containing variable placeholders (e.g., `${var_name}`), extracting variables should return all unique variable names present in the template.

**Validates: Requirements 8.3**

### Property 13: Credentials Security Invariant

*For any* Credentials object associated with SSH key authentication, the credentials must contain only the key file path and never the actual private key content.

**Validates: Requirements 5.5**

### Property 14: Group Hierarchy Integrity

*For any* sequence of group creation and nesting operations, the resulting hierarchy should be acyclic (no group is its own ancestor) and all parent references should point to existing groups.

**Validates: Requirements 1.4**

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum RustConnError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Secret storage error: {0}")]
    Secret(#[from] SecretError),
    
    #[error("Import error: {0}")]
    Import(#[from] ImportError),
    
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to parse configuration: {0}")]
    Parse(String),
    
    #[error("Invalid configuration value for {field}: {reason}")]
    Validation { field: String, reason: String },
    
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),
    
    #[error("Failed to write configuration: {0}")]
    Write(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    
    #[error("Client not found: {0}")]
    ClientNotFound(PathBuf),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Failed to parse {source}: {reason}")]
    ParseError { source: String, reason: String },
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}
```

### Error Handling Strategy

1. **User-Facing Errors**: Display via GTK4 `AlertDialog` with actionable messages
2. **Recoverable Errors**: Log warning, continue operation where possible (e.g., import)
3. **Fatal Errors**: Log error, display message, graceful shutdown
4. **Async Errors**: Propagate via `Result`, handle in UI layer

## Testing Strategy

### Property-Based Testing Framework

The project will use **proptest** crate for property-based testing in Rust.

```toml
[dev-dependencies]
proptest = "1.4"
```

### Test Configuration

- Minimum 100 iterations per property test
- Each property test must be annotated with the correctness property it validates
- Format: `// **Feature: rustconn, Property {N}: {property_name}**`

### Unit Tests

Unit tests will cover:
- Edge cases for parsers (empty input, malformed data)
- Error condition handling
- Individual component behavior

### Integration Tests

Integration tests will cover:
- Full import workflows with real file formats
- Secret storage backend integration
- GTK4 widget behavior (where feasible)

### Test Organization

```
tests/
├── properties/
│   ├── connection_tests.rs      # Properties 1, 2, 14
│   ├── serialization_tests.rs   # Property 6
│   ├── command_builder_tests.rs # Properties 3, 4, 5
│   ├── import_tests.rs          # Properties 7, 8, 9, 10, 11
│   ├── snippet_tests.rs         # Property 12
│   └── security_tests.rs        # Property 13
├── unit/
│   ├── config_tests.rs
│   ├── protocol_tests.rs
│   └── parser_tests.rs
└── integration/
    ├── import_integration.rs
    └── secret_storage.rs
```

### Generator Strategy for Property Tests

```rust
// Example: Connection generator
prop_compose! {
    fn arb_connection()(
        name in "[a-zA-Z][a-zA-Z0-9_-]{0,63}",
        host in "[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*",
        port in 1u16..65535,
        protocol in prop_oneof![
            Just(ProtocolType::Ssh),
            Just(ProtocolType::Rdp),
            Just(ProtocolType::Vnc),
        ],
        tags in prop::collection::vec("[a-z]{1,20}", 0..5),
    ) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            name,
            host,
            port,
            protocol,
            tags,
            // ... other fields with defaults
        }
    }
}
```
