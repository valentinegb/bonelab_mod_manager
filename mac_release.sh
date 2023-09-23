#!/bin/bash

# Build for an Apple Silicon Mac
cargo build --release --target aarch64-apple-darwin

# Build for an Intel Mac
cargo build --release --target x86_64-apple-darwin

# Remove bundle if it already exists
rm -rf "target/universal-apple-darwin/release/Bonelab Mod Manager.app"

# Create macOS application bundle
mkdir -p \
    "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/Resources" \
    "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/MacOS"

# Merge two executables into universal exectuable
lipo \
    target/aarch64-apple-darwin/release/bonelab_mod_manager \
    target/x86_64-apple-darwin/release/bonelab_mod_manager \
    -create \
    -output "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/Resources/bonelab_mod_manager"

# Create script to open terminal when app is run
cat <<EOF > "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/MacOS/Bonelab Mod Manager"
#!/bin/bash
/System/Applications/Utilities/Terminal.app/Contents/MacOS/Terminal "\$(dirname "\$0")/../Resources/bonelab_mod_manager"
EOF

# Make script executable
chmod +x "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/MacOS/Bonelab Mod Manager"

# Copy app icon into bundle
cp AppIcon.png "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/Resources/AppIcon.png"

# Create Info.plist
cat <<EOF > "target/universal-apple-darwin/release/Bonelab Mod Manager.app/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>Bonelab Mod Manager</string>
	<key>CFBundleDisplayName</key>
	<string>Bonelab Mod Manager</string>
	<key>CFBundleIdentifier</key>
	<string>com.valentinegb.bonelab_mod_manager</string>
	<key>CFBundleVersion</key>
	<string>0.5.0</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleSignature</key>
	<string>bbmm</string>
	<key>CFBundleExecutable</key>
	<string>Bonelab Mod Manager</string>
	<key>CFBundleIconFile</key>
	<string>AppIcon</string>
	<key>CFBundleIconName</key>
	<string>AppIcon</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.utilities</string>
</dict>
</plist>
EOF
