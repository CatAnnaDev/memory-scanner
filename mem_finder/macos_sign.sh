#!/bin/bash

# Configuration
# security find-identity -v -p codesigning
BINARY="target/release/mem_finder"
IDENTITY="Apple Development: mail@mail.com (TEAM)"
ENTITLEMENTS="entitlements.plist"

# Compiler
echo "🏗️  Compilation..."
cargo build --release

# Signer
echo "✍️  Signature..."
codesign --force --sign "$IDENTITY" \
  --entitlements "$ENTITLEMENTS" \
  "$BINARY"
ß
# Vérifier
echo "✅ Vérification..."
codesign -d -vvv "$BINARY"
codesign -d --entitlements - "$BINARY"

echo "✨ Terminé !"