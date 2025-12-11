# RustConn Product Overview

RustConn is a modern connection manager for Linux, designed to manage SSH, RDP, and VNC remote connections through a GTK4-based GUI with Wayland-native support.

## Core Features

- Multi-protocol support: SSH (embedded terminal), RDP, and VNC (external windows)
- Connection organization with groups and tags
- Import from existing tools: Asbru-CM, Remmina, SSH config, Ansible inventory
- Secure credential storage via KeePassXC and libsecret integration
- Session management with logging capabilities
- Command snippets with variable substitution

## Target Users

Linux system administrators and developers who manage multiple remote connections and want a native, modern alternative to tools like Asbru-CM or Remmina.

## Design Philosophy

- Wayland-first, GTK4-native interface
- Security-conscious: no plaintext credential storage, uses system secret services
- Extensible protocol and import system via traits
- Clean separation between core library and GUI application
