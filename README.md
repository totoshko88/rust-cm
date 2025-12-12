# RustConn

A modern connection manager for Linux, designed to manage SSH, RDP, and VNC remote connections through a GTK4-based GUI with Wayland-native support.

![RustConn Screenshot](rustconn/assets/rustconn.svg)

## Features

- **Multi-protocol support**: SSH (embedded terminal), RDP, and VNC
- **Connection organization**: Groups and tags for easy management
- **Import from existing tools**: Asbru-CM, Remmina, SSH config, Ansible inventory
- **Secure credential storage**: KeePassXC and libsecret integration
- **Session management**: Logging capabilities and session tracking
- **Command snippets**: Variable substitution for common commands
- **Split view**: Multiple terminals in split panes
- **Wayland-first**: Native GTK4 interface

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/totoshko88/rustconn.git
cd rustconn

# Build release version
cargo build --release

# Install (optional)
./install-desktop.sh
```

### Dependencies

- GTK4 (4.12+)
- VTE4 (for terminal emulation)
- FreeRDP (xfreerdp/xfreerdp3) for RDP connections
- TigerVNC or TightVNC for VNC connections

## Usage

```bash
# Run from source
cargo run -p rustconn

# Or after installation
rustconn
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New connection |
| `Ctrl+I` | Import connections |
| `Ctrl+,` | Settings |
| `Ctrl+Shift+S` | Split vertical |
| `Ctrl+Shift+H` | Split horizontal |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |

## Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check lints
cargo clippy --all-targets
```

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Support the Project

If you find RustConn useful, consider supporting its development:

- **Ko-Fi**: [https://ko-fi.com/totoshko88](https://ko-fi.com/totoshko88)
- **PayPal/Payoneer**: totoshko88@gmail.com
- **UAH (Monobank)**: [https://send.monobank.ua/jar/2UgaGcQ3JC](https://send.monobank.ua/jar/2UgaGcQ3JC)

## Acknowledgments

- GTK4 and the GNOME project
- The Rust community
- FreeRDP project
- All contributors and supporters

---

Made with ❤️ in Ukraine 🇺🇦
