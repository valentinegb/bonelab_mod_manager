#!/bin/bash

# Build for an Apple Silicon Mac
cargo build --release --target aarch64-apple-darwin

# Build for an Intel Mac
cargo build --release --target x86_64-apple-darwin

# Create folder to put universal executable into
mkdir -p target/universal-apple-darwin/release/bonelab_mod_manager_dmg

# Merge two executables into universal exectuable
lipo \
    target/aarch64-apple-darwin/release/bonelab_mod_manager \
    target/x86_64-apple-darwin/release/bonelab_mod_manager \
    -create \
    -output target/universal-apple-darwin/release/bonelab_mod_manager_dmg/bonelab_mod_manager

# Remove DMG, in case it already exists
rm -f target/universal-apple-darwin/release/bonelab_mod_manager.dmg

# Make DMG from folder
hdiutil \
    create \
    -srcfolder target/universal-apple-darwin/release/bonelab_mod_manager_dmg \
    target/universal-apple-darwin/release/bonelab_mod_manager.dmg

# Remove folder
rm -rf target/universal-apple-darwin/release/bonelab_mod_manager_dmg
