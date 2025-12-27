#!/bin/bash
# Script to add RustConn exception to flatpak-builder-lint
# Run this from a directory where you want to clone the repo

set -e

REPO_URL="https://github.com/flathub-infra/flatpak-builder-lint.git"
FORK_URL="https://github.com/totoshko88/flatpak-builder-lint.git"
BRANCH_NAME="add-rustconn-exception"

echo "=== Adding RustConn exception to flatpak-builder-lint ==="

# Check if we already have a clone
if [ -d "flatpak-builder-lint" ]; then
    echo "Directory exists, updating..."
    cd flatpak-builder-lint
    git fetch origin
    git checkout main
    git pull origin main
else
    echo "Cloning repository..."
    # First fork the repo on GitHub, then clone your fork
    git clone "$FORK_URL"
    cd flatpak-builder-lint
    git remote add upstream "$REPO_URL"
    git fetch upstream
fi

# Create branch
git checkout -b "$BRANCH_NAME" || git checkout "$BRANCH_NAME"

# The exception file
EXCEPTIONS_FILE="flatpak_builder_lint/staticfiles/exceptions.json"

echo "Adding RustConn exception..."

# Use Python to add the exception properly (maintains JSON formatting)
python3 << 'EOF'
import json

exceptions_file = "flatpak_builder_lint/staticfiles/exceptions.json"

with open(exceptions_file, 'r') as f:
    exceptions = json.load(f)

# Add RustConn exception
exceptions["io.github.totoshko88.RustConn"] = {
    "finish-args-ssh-filesystem-access": "RustConn is an SSH/RDP/VNC/SPICE connection manager. Read-only access to ~/.ssh is required to load user's existing SSH keys and config for SSH connections.",
    "finish-args-has-socket-ssh-auth": "Required for SSH agent forwarding to enable key-based authentication for SSH connections without re-entering passphrases."
}

# Sort by keys and write back
sorted_exceptions = dict(sorted(exceptions.items()))

with open(exceptions_file, 'w') as f:
    json.dump(sorted_exceptions, f, indent=4)
    f.write('\n')

print("Exception added successfully!")
EOF

# Commit
git add "$EXCEPTIONS_FILE"
git commit -m "Add exception for io.github.totoshko88.RustConn

RustConn is an SSH/RDP/VNC/SPICE connection manager that requires:
- Read-only access to ~/.ssh for loading SSH keys and config
- SSH agent socket for key-based authentication

Similar to other SSH clients on Flathub:
- io.github.mfat.sshpilot
- io.github.BuddySirJava.SSH-Studio
- com.github.muriloventuroso.easyssh"

echo ""
echo "=== Done! ==="
echo "Now push and create PR:"
echo "  git push -u origin $BRANCH_NAME"
echo "  gh pr create --repo flathub-infra/flatpak-builder-lint --title 'Add exception for io.github.totoshko88.RustConn' --body 'RustConn is an SSH/RDP/VNC/SPICE connection manager...'"
