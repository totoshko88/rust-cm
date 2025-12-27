#!/bin/bash
# Update OBS package with new version
# Usage: ./scripts/update-obs.sh [version]

set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

echo "Updating OBS package to version $VERSION"

OBS_DIR="home:totoshko88:rustconn/rustconn"

# Check if OBS checkout exists
if [ ! -d "$OBS_DIR" ]; then
    echo "Error: OBS checkout not found at $OBS_DIR"
    echo "Run: osc checkout home:totoshko88:rustconn/rustconn"
    exit 1
fi

# Update _service revision
echo "Updating _service revision to v$VERSION..."
sed -i "s|<param name=\"revision\">v[^<]*</param>|<param name=\"revision\">v$VERSION</param>|" "$OBS_DIR/_service"

# Update spec version
echo "Updating rustconn.spec version..."
sed -i "s/^Version:.*$/Version:        $VERSION/" "$OBS_DIR/rustconn.spec"

# Copy updated changelog
echo "Copying rustconn.changes..."
cp packaging/obs/rustconn.changes "$OBS_DIR/rustconn.changes"

# Copy updated spec (with changelog section)
echo "Copying rustconn.spec..."
cp packaging/obs/rustconn.spec "$OBS_DIR/rustconn.spec"

# Copy vendor tarball if exists
if [ -f "vendor.tar.zst" ]; then
    echo "Copying vendor.tar.zst..."
    cp vendor.tar.zst "$OBS_DIR/vendor.tar.zst"
fi

echo ""
echo "Files updated. To commit to OBS:"
echo "  cd $OBS_DIR"
echo "  osc status"
echo "  osc commit -m 'Update to version $VERSION'"
