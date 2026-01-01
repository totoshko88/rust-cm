# Installation Guide

## Flatpak

```bash
# Install from Flatpak bundle
flatpak install RustConn-0.5.3.flatpak

# Run
flatpak run io.github.totoshko88.RustConn
```

## AppImage

```bash
chmod +x RustConn-0.5.3-x86_64.AppImage
./RustConn-0.5.3-x86_64.AppImage
```

## Debian/Ubuntu

```bash
sudo dpkg -i rustconn_0.5.3-1_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

## openSUSE (OBS)

```bash
# Tumbleweed
sudo zypper ar https://download.opensuse.org/repositories/home:/totoshko88:/rustconn/openSUSE_Tumbleweed/ rustconn
sudo zypper ref
sudo zypper in rustconn

# Leap 16.0
sudo zypper ar https://download.opensuse.org/repositories/home:/totoshko88:/rustconn/openSUSE_Leap_16.0/ rustconn
sudo zypper ref
sudo zypper in rustconn
```

## From Source

### Prerequisites

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libvte-2.91-gtk4-dev libadwaita-1-dev libdbus-1-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel vte291-gtk4-devel libadwaita-devel dbus-devel
```

**openSUSE:**
```bash
sudo zypper install gtk4-devel vte-devel libadwaita-devel dbus-1-devel
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 vte4 libadwaita dbus
```

### Build

```bash
git clone https://github.com/totoshko88/rustconn.git
cd rustconn
cargo build --release
```

### Install (optional)

```bash
./install-desktop.sh
```

## Dependencies

### Required
- GTK4 (4.14+)
- VTE4 (terminal emulation)
- libadwaita

### Optional (for protocol support)
- **RDP:** FreeRDP (`xfreerdp` or `xfreerdp3`)
- **VNC:** TigerVNC (`vncviewer`) or TightVNC
- **SPICE:** `remote-viewer` (virt-viewer package)

### Zero Trust CLI tools
Each Zero Trust provider requires its CLI:
- **AWS SSM:** `aws` CLI with SSM plugin
- **GCP IAP:** `gcloud` CLI
- **Azure:** `az` CLI
- **Tailscale:** `tailscale` CLI
- **Teleport:** `tsh` CLI

## Rust Version

RustConn requires Rust 1.87+ (MSRV). Install via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```
