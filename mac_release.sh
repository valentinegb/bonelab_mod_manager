#!/bin/bash

# Build for an Apple Silicon Mac
cargo build --release --target aarch64-apple-darwin

# Create folder to put build into
mkdir target/aarch64-apple-darwin/release/bonelab_mod_manager_dmg

# Put build into folder
cp target/aarch64-apple-darwin/release/bonelab_mod_manager target/aarch64-apple-darwin/release/bonelab_mod_manager_dmg/bonelab_mod_manager

# Make DMG from folder
hdiutil create -srcfolder target/aarch64-apple-darwin/release/bonelab_mod_manager_dmg target/aarch64-apple-darwin/release/bonelab_mod_manager.dmg

# Remove folder
rm -rf target/aarch64-apple-darwin/release/bonelab_mod_manager_dmg
