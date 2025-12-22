#
# spec file for package rustconn
#
# Copyright (c) 2025 Anton Isaiev
# SPDX-License-Identifier: MIT
#

Name:           rustconn
Version:        0.1.0
Release:        0
Summary:        Modern connection manager for Linux (SSH, RDP, VNC, SPICE)
License:        MIT
URL:            https://github.com/totoshko88/rust-cm
Source0:        %{name}-%{version}.tar.xz
Source1:        vendor.tar.zst

BuildRequires:  cargo >= 1.76
BuildRequires:  rust >= 1.76
BuildRequires:  cargo-packaging
BuildRequires:  pkgconfig(gtk4) >= 4.14
BuildRequires:  pkgconfig(vte-2.91-gtk4)
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  pkgconfig(openssl)
BuildRequires:  zstd

# Runtime dependencies
Requires:       gtk4 >= 4.14
Requires:       vte >= 0.74
Requires:       openssh-clients

# Optional runtime dependencies
Recommends:     freerdp
Recommends:     tigervnc
Recommends:     virt-viewer

%description
RustConn is a modern connection manager for Linux desktops.
It provides a GTK4-based GUI for managing SSH, RDP, VNC, and SPICE
remote connections with features like connection groups, credential
management, session logging, and import/export capabilities.

%prep
%autosetup -a1 -n %{name}-%{version}
mkdir -p .cargo
cat > .cargo/config.toml <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
%{cargo_build} --release -p rustconn -p rustconn-cli

%install
install -Dm755 target/release/rustconn %{buildroot}%{_bindir}/rustconn
install -Dm755 target/release/rustconn-cli %{buildroot}%{_bindir}/rustconn-cli
install -Dm644 rustconn/assets/org.rustconn.RustConn.desktop %{buildroot}%{_datadir}/applications/org.rustconn.RustConn.desktop

# Install icons
for size in 16 24 32 48 64 128 256 512; do
    if [ -f "rustconn/assets/icons/hicolor/${size}x${size}/apps/rustconn.png" ]; then
        install -Dm644 "rustconn/assets/icons/hicolor/${size}x${size}/apps/rustconn.png" \
            "%{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/rustconn.png"
    fi
done

if [ -f "rustconn/assets/icons/hicolor/scalable/apps/rustconn.svg" ]; then
    install -Dm644 "rustconn/assets/icons/hicolor/scalable/apps/rustconn.svg" \
        "%{buildroot}%{_datadir}/icons/hicolor/scalable/apps/rustconn.svg"
fi

%files
%license LICENSE
%doc README.md CHANGELOG.md docs/
%{_bindir}/rustconn
%{_bindir}/rustconn-cli
%{_datadir}/applications/org.rustconn.RustConn.desktop
%{_datadir}/icons/hicolor/*/apps/rustconn.*

%changelog
