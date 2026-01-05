//! Clients detection tab

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ScrolledWindow};
use rustconn_core::protocol::ClientDetectionResult;
use std::path::PathBuf;

/// Creates the clients detection tab
pub fn create_clients_tab() -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();

    let main_vbox = GtkBox::new(Orientation::Vertical, 6);
    main_vbox.set_margin_top(12);
    main_vbox.set_margin_bottom(12);
    main_vbox.set_margin_start(12);
    main_vbox.set_margin_end(12);
    main_vbox.set_valign(gtk4::Align::Start);

    // Detect all clients
    let detection_result = ClientDetectionResult::detect_all();

    // SSH Client
    let ssh_section = create_protocol_section(
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
    let rdp_section = create_protocol_section(
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
    let vnc_section = create_protocol_section(
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
    let spice_section = create_spice_section();

    // Zero Trust Clients
    let zerotrust_section = create_zerotrust_section();

    main_vbox.append(&ssh_section);
    main_vbox.append(&rdp_section);
    main_vbox.append(&vnc_section);
    main_vbox.append(&spice_section);
    main_vbox.append(&zerotrust_section);

    scrolled.set_child(Some(&main_vbox));
    scrolled
}

/// Creates a protocol client section with consistent layout
fn create_protocol_section(
    title: &str,
    name: &str,
    installed: bool,
    version: Option<&str>,
    path: Option<&str>,
    install_hint: Option<&str>,
) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 2);
    section.set_margin_bottom(12);

    // Header
    let header = Label::builder()
        .label(title)
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    section.append(&header);

    if installed {
        // Version row
        if let Some(ver) = version {
            let version_row = create_info_row("Version:", ver);
            section.append(&version_row);
        }

        // Path row
        if let Some(p) = path {
            let path_row = create_info_row("Path:", p);
            section.append(&path_row);
        }

        // Status
        let status_label = Label::builder()
            .label(&format!("✓ {name} detected"))
            .halign(gtk4::Align::Start)
            .css_classes(["success"])
            .margin_start(6)
            .margin_top(2)
            .build();
        section.append(&status_label);
    } else {
        // Not installed status with hint on the right
        let status_row = GtkBox::new(Orientation::Horizontal, 6);
        status_row.set_margin_start(6);
        status_row.set_margin_top(4);

        let status_label = Label::builder()
            .label(&format!("✗ {title} not found"))
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["error"])
            .build();
        status_row.append(&status_label);

        if let Some(hint) = install_hint {
            let hint_label = Label::builder()
                .label(hint)
                .halign(gtk4::Align::End)
                .css_classes(["dim-label"])
                .build();
            status_row.append(&hint_label);
        }

        section.append(&status_row);
    }

    section
}

/// Creates an info row with label on left and value on right
fn create_info_row(label: &str, value: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.set_margin_start(6);

    let label_widget = Label::builder()
        .label(label)
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .build();

    let value_widget = Label::builder()
        .label(value)
        .halign(gtk4::Align::End)
        .hexpand(true)
        .selectable(true)
        .ellipsize(gtk4::pango::EllipsizeMode::Start)
        .build();

    row.append(&label_widget);
    row.append(&value_widget);
    row
}

/// Creates the SPICE client section
fn create_spice_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 2);
    section.set_margin_bottom(12);

    let header = Label::builder()
        .label("SPICE Client")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    section.append(&header);

    let spice_installed = std::process::Command::new("which")
        .arg("remote-viewer")
        .output()
        .is_ok_and(|output| output.status.success());

    if spice_installed {
        // Path first
        let spice_path = if let Ok(output) = std::process::Command::new("which")
            .arg("remote-viewer")
            .output()
        {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        };

        if let Some(ref path) = spice_path {
            let path_row = create_info_row("Path:", path);
            section.append(&path_row);
        }

        // Version - use the common parser
        if let Some(ref path) = spice_path {
            if let Some(version) = get_version(std::path::Path::new(path), "--version") {
                let version_row = create_info_row("Version:", &version);
                section.append(&version_row);
            }
        }

        let status_label = Label::builder()
            .label("✓ remote-viewer detected")
            .halign(gtk4::Align::Start)
            .css_classes(["success"])
            .margin_start(6)
            .margin_top(2)
            .build();
        section.append(&status_label);
    } else {
        let status_row = GtkBox::new(Orientation::Horizontal, 6);
        status_row.set_margin_start(6);
        status_row.set_margin_top(4);

        let status_label = Label::builder()
            .label("✗ SPICE client not found")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["error"])
            .build();
        status_row.append(&status_label);

        let hint_label = Label::builder()
            .label("Install virt-viewer package")
            .halign(gtk4::Align::End)
            .css_classes(["dim-label"])
            .build();
        status_row.append(&hint_label);

        section.append(&status_row);
    }

    section
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

    // Try stdout first, then stderr
    let version_str = if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };

    // Get command name for specific parsing
    let cmd_name = command_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    parse_version_output(cmd_name, &version_str)
}

/// Parses version from command output based on command type
fn parse_version_output(command: &str, output: &str) -> Option<String> {
    match command {
        // aws-cli/2.32.28 Python/3.13.11 Linux/... -> 2.32.28
        "aws" => output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .and_then(|part| part.strip_prefix("aws-cli/"))
            .map(String::from),

        // Google Cloud SDK 550.0.0 -> 550.0.0
        "gcloud" => output.lines().find_map(|line| {
            line.strip_prefix("Google Cloud SDK ")
                .map(|v| v.trim().to_string())
        }),

        // azure-cli                         2.81.0 -> 2.81.0
        "az" => output.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("azure-cli") {
                trimmed.split_whitespace().last().map(String::from)
            } else {
                None
            }
        }),

        // cloudflared version 2025.11.1 (built ...) -> 2025.11.1
        "cloudflared" => output.lines().next().and_then(|line| {
            line.split_whitespace()
                .nth(2)
                .map(|v| v.trim_end_matches(['(', ' ']).to_string())
        }),

        // Teleport vX.X.X git:... -> vX.X.X
        "teleport" => output.lines().next().and_then(|line| {
            line.split_whitespace()
                .nth(1)
                .map(|v| v.trim_start_matches('v').to_string())
        }),

        // Version Number:      0.21.0 -> 0.21.0
        "boundary" => output.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("Version Number:") {
                trimmed.split(':').nth(1).map(|s| s.trim().to_string())
            } else {
                None
            }
        }),

        // tailscale --version outputs just version number: "1.92.3"
        "tailscale" => output
            .lines()
            .next()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty()),

        // oci --version outputs just version number: "3.71.4"
        "oci" => output
            .lines()
            .next()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty()),

        // remote-viewer, версія 11.0 -> 11.0
        "remote-viewer" => output.lines().next().and_then(|line| {
            // Try to find version number after comma
            line.split(',')
                .nth(1)
                .and_then(|part| part.split_whitespace().last())
                .map(String::from)
        }),

        // Default: return first non-empty line
        _ => output
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string()),
    }
}

/// Creates the Zero Trust clients section
fn create_zerotrust_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 4);
    section.set_margin_bottom(12);

    let header = Label::builder()
        .label("Zero Trust Clients")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    section.append(&header);

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
        let client_row = create_zerotrust_client_row(name, command, version_arg, install_hint);
        section.append(&client_row);
    }

    section
}

/// Creates a row for a Zero Trust client
fn create_zerotrust_client_row(
    name: &str,
    command: &str,
    version_arg: &str,
    install_hint: &str,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Vertical, 2);
    row.set_margin_top(6);
    row.set_margin_start(6);

    // Try to find the command
    let command_path = find_command(command);
    let installed = command_path.is_some();

    if installed {
        let path = command_path.as_ref().expect("checked above");

        // Status line with name
        let status_label = Label::builder()
            .label(&format!("✓ {name}"))
            .halign(gtk4::Align::Start)
            .css_classes(["success"])
            .build();
        row.append(&status_label);

        // Path row
        let path_row = create_info_row("Path:", &path.display().to_string());
        row.append(&path_row);

        // Version row
        if let Some(version) = get_version(path, version_arg) {
            let version_row = create_info_row("Version:", &version);
            row.append(&version_row);
        }
    } else {
        // Not installed - status with hint on right
        let status_row = GtkBox::new(Orientation::Horizontal, 6);

        let status_label = Label::builder()
            .label(&format!("✗ {name}"))
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["error"])
            .build();
        status_row.append(&status_label);

        let hint_label = Label::builder()
            .label(install_hint)
            .halign(gtk4::Align::End)
            .css_classes(["dim-label"])
            .build();
        status_row.append(&hint_label);

        row.append(&status_row);
    }

    row
}
