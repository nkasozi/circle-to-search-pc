#!/bin/bash

set -e

APP_NAME="Circle to Search"
EXECUTABLE_NAME="circle-to-search-pc"
INSTALL_DIR="$HOME/.local/share/circle-to-search-pc"
BIN_DIR="$HOME/.local/bin"
DESKTOP_FILE_DIR="$HOME/.local/share/applications"

echo "========================================"
echo "  Building $APP_NAME for Linux"
echo "========================================"
echo ""

echo "[1/7] Checking prerequisites..."
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found. Please install from https://rustup.rs"
    exit 1
fi
echo "  ✓ Cargo found"

if [ ! -f "tessdata/eng.traineddata" ]; then
    echo "ERROR: tessdata not found. Please run ./download-tessdata.sh first"
    exit 1
fi
echo "  ✓ tessdata found"

echo ""
echo "[2/7] Building release binary..."
cargo build --release
echo "  ✓ Build successful"

echo ""
echo "[3/7] Stopping any running instance..."
if pkill -x "$EXECUTABLE_NAME" 2>/dev/null; then
    sleep 1
    echo "  ✓ Stopped existing instance"
else
    echo "  ✓ No running instance found"
fi

echo ""
echo "[4/7] Creating installation directories..."
if [ -d "$INSTALL_DIR" ]; then
    rm -rf "$INSTALL_DIR"
    echo "  ✓ Removed old installation"
fi
mkdir -p "$INSTALL_DIR/tessdata"
mkdir -p "$BIN_DIR"
mkdir -p "$DESKTOP_FILE_DIR"
echo "  ✓ Created: $INSTALL_DIR"

echo ""
echo "[5/7] Copying files..."
cp "target/release/$EXECUTABLE_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$EXECUTABLE_NAME"
echo "  ✓ Copied executable"

cp -r tessdata/* "$INSTALL_DIR/tessdata/"
echo "  ✓ Copied tessdata"

if [ -f "assets/icons/icon.png" ]; then
    cp "assets/icons/icon.png" "$INSTALL_DIR/"
    echo "  ✓ Copied icon"
fi

echo ""
echo "[6/7] Creating symlink in ~/.local/bin..."
ln -sf "$INSTALL_DIR/$EXECUTABLE_NAME" "$BIN_DIR/$EXECUTABLE_NAME"
echo "  ✓ Created symlink: $BIN_DIR/$EXECUTABLE_NAME"

echo ""
echo "[7/7] Creating desktop entry..."
cat > "$DESKTOP_FILE_DIR/circle-to-search.desktop" << EOF
[Desktop Entry]
Name=$APP_NAME
Comment=Screen capture with OCR and Google Lens search
Exec=$INSTALL_DIR/$EXECUTABLE_NAME
Icon=$INSTALL_DIR/icon.png
Terminal=false
Type=Application
Categories=Utility;Graphics;
Keywords=screenshot;ocr;search;lens;capture;
StartupNotify=true
EOF

chmod +x "$DESKTOP_FILE_DIR/circle-to-search.desktop"
echo "  ✓ Created desktop entry"

if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_FILE_DIR" 2>/dev/null || true
fi

echo ""
echo "========================================"
echo "  Installation Complete!"
echo "========================================"
echo ""
echo "Installed to: $INSTALL_DIR"
echo ""
echo "To launch, either:"
echo "  1. Search 'Circle to Search' in your application menu"
echo "  2. Run: $EXECUTABLE_NAME (if ~/.local/bin is in your PATH)"
echo "  3. Run: $INSTALL_DIR/$EXECUTABLE_NAME"
echo ""
echo "Note: If '$EXECUTABLE_NAME' command is not found, add this to your ~/.bashrc or ~/.zshrc:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "To uninstall, simply delete:"
echo "  - $INSTALL_DIR"
echo "  - $BIN_DIR/$EXECUTABLE_NAME"
echo "  - $DESKTOP_FILE_DIR/circle-to-search.desktop"
echo ""
