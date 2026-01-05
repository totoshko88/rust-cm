//! Clients detection tab using libadwaita components

use adw::prelude::*;
use gtk4::Label;
use libadwaita as adw;
use rustconn_core::protocol::ClientDetectionResult;
use std::path::PathBuf;

/// Creates the clients detection page using AdwPreferencesPage
pub fn create_clients_page() -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title("Clients")
        .icon_name("preferences-system-symbolic")
        .build();

    // Detect all clients
    let detection_result = ClientDetectionResult::detect_all();

    // === Core Clients Group ===
    let core_group = adw::PreferencesGroup::builder()
        .title("Core Clients")
        .description("Essential connection clients")
        .build();

    // SSH Client
    add_client_row(
        &core_group,
        "SSH Client",
        &detection_result.ssh.name,
        detection_result.ssh.installed,
        detection_result.ssh.version.as_deref(),
        detection_result
            .ssh
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .as_deref(),
        detection_result.ssh.install_hint.as_deref(),
    );

    // RDP Client
    add_client_row(
        &core_group,
        "RDP Client",
        &detection_result.rdp.name,
        detection_result.rdp.installed,
        detection_result.rdp.version.as_deref(),
        detection_result
            .rdp
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .as_deref(),
        detection_result.rdp.install_hint.as_deref(),
    );

    // VNC Client
    add_client_row(
        &core_group,
        "VNC Client",
        &detection_result.vnc.name,
        detection_result.vnc.installed,
        detection_result.vnc.version.as_deref(),
        detection_result
            .vnc
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .as_deref(),
        detection_result.vnc.install_hint.as_deref(),
    );

    // SPICE Client
    let spice_installed = std::process::Command::new("which")
        .arg("remote-viewer")
        .output()
        .is_ok_and(|output| output.status.success());

    let spice_path = if spice_installed {
        std::process::Command::new("which")
            .arg("remote-viewer")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    let spice_version = spice_path
        .as_ref()
        .and_then(|p| get_version(std::path::Path::new(p), "--version"));

    add_client_row(
        &core_group,
        "SPICE Client",
        "remote-viewer",
        spice_installed,
        spice_version.as_deref(),
        spice_path.as_deref(),
        Some("Install virt-viewer package"),
    );

    page.add(&core_group);

    // === Zero Trust Clients Group ===
    let zerotrust_group = adw::PreferencesGroup::builder()
        .title("Zero Trust Clients")
        .description("Cloud provider CLI tools")
        .build();

    let zerotrust_clients = [
        (
            "AWS CLI (SSM)",
            "aws",
            "--version",
            "Install awscli package",
        ),
        (
            "Google Cloud CLI",
            "gcloud",
            "--version",
            "Install google-cloud-cli package",
        ),
        ("Azure CLI", "az", "--version", "Install azure-cli package"),
        ("OCI CLI", "oci", "--version", "Install oci-cli package"),
        (
            "Cloudflare CLI",
            "cloudflared",
            "--version",
            "Install cloudflared package",
        ),
        (
            "Teleport CLI",
            "teleport",
            "version",
            "Install teleport package",
        ),
        (
            "Tailscale CLI",
            "tailscale",
            "--version",
            "Install tailscale package",
        ),
        ("Boundary CLI", "boundary", "-v", "Install boundary package"),
    ];

    for (name, command, version_arg, install_hint) in &zerotrust_clients {
        let command_path = find_command(command);
        let installed = command_path.is_some();

        let version = command_path
            .as_ref()
            .and_then(|p| get_version(p, version_arg));
        let path_str = command_path.as_ref().map(|p| p.display().to_string());

        add_client_row(
            &zerotrust_group,
            name,
            command,
            installed,
            version.as_deref(),
            path_str.as_deref(),
            Some(install_hint),
        );
    }

    page.add(&zerotrust_group);

    page
}

/// Adds a client row to a preferences group
fn add_client_row(
    group: &adw::PreferencesGroup,
    title: &str,
    name: &str,
    installed: bool,
    version: Option<&str>,
    path: Option<&str>,
    install_hint: Option<&str>,
) {
    // Path goes in subtitle (left-aligned), version goes in suffix (right-aligned)
    let subtitle = if installed {
        path.map(String::from).unwrap_or_else(|| name.to_string())
    } else {
        install_hint.unwrap_or("Not installed").to_string()
    };

    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(&subtitle)
        .build();

    // Status icon (checkmark or X)
    let status_label = Label::builder()
        .label(if installed { "✓" } else { "✗" })
        .valign(gtk4::Align::Center)
        .css_classes([if installed { "success" } else { "error" }])
        .build();
    row.add_prefix(&status_label);

    // Version label on the right (suffix)
    if installed {
        if let Some(v) = version {
            let version_label = Label::builder()
                .label(&format!("v{v}"))
                .valign(gtk4::Align::Center)
                .css_classes(["dim-label"])
                .build();
            row.add_suffix(&version_label);
        }
    }

    group.add(&row);
}

/// Finds a command in PATH or common user directories
fn find_command(command: &str) -> Option<PathBuf> {
    // First try standard which
    if let Ok(output) = std::process::Command::new("which").arg(command).output() {
        if output.status.success() {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    // Check common user directories
    if let Some(home) = dirs::home_dir() {
        let user_paths = [
            home.join("bin").join(command),
            home.join(".local/bin").join(command),
            home.join(".cargo/bin").join(command),
        ];

        for path in &user_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }
    }

    None
}

/// Gets version output from a command and parses it
fn get_version(command_path: &std::path::Path, version_arg: &str) -> Option<String> {
    let output = std::process::Command::new(command_path)
        .arg(version_arg)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let version_str = if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };

    let cmd_name = command_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    parse_version_output(cmd_name, &version_str)
}

/// Parses version from command output based on command type
fn parse_version_output(command: &str, output: &str) -> Option<String> {
    match command {
        "aws" => output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .and_then(|part| part.strip_prefix("aws-cli/"))
            .map(String::from),

        "gcloud" => output.lines().find_map(|line| {
            line.strip_prefix("Google Cloud SDK ")
                .map(|v| v.trim().to_string())
        }),

        "az" => output.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("azure-cli") {
                trimmed.split_whitespace().last().map(String::from)
            } else {
                None
            }
        }),

        "cloudflared" => output.lines().next().and_then(|line| {
            line.split_whitespace()
                .nth(2)
                .map(|v| v.trim_end_matches(['(', ' ']).to_string())
        }),

        "teleport" => output.lines().next().and_then(|line| {
            line.split_whitespace()
                .nth(1)
                .map(|v| v.trim_start_matches('v').to_string())
        }),

        "boundary" => output.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("Version Number:") {
                trimmed.split(':').nth(1).map(|s| s.trim().to_string())
            } else {
                None
            }
        }),

        "tailscale" => output
            .lines()
            .next()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty()),

        "oci" => output
            .lines()
            .next()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty()),

        "remote-viewer" => output.lines().next().and_then(|line| {
            line.split(',')
                .nth(1)
                .and_then(|part| part.split_whitespace().last())
                .map(String::from)
        }),

        _ => output
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string()),
    }
}
