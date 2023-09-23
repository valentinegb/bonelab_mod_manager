# Bonelab Mod Manager [![Rust](https://github.com/valentinegb/bonelab_mod_manager/actions/workflows/rust.yml/badge.svg)](https://github.com/valentinegb/bonelab_mod_manager/actions/workflows/rust.yml)

Keeps your installed Bonelab mods synced with your subscribed mod.io mods.

<img width="858" alt="Screenshot 2023-09-08 at 4 38 43 PM" src="https://github.com/valentinegb/bonelab_mod_manager/assets/35977727/b2c810b3-586b-4103-ad96-f1bc449abf30">

## Features

- Multi-platform
  - macOS (Apple Silicon and Intel)
  - Linux (x64)
  - Windows (x64)
- PC and Quest mod support
- Parallel downloads
- Installs subscribed mod.io mods
- Installs PC mods directly into Bonelab installation
- Updates mods
- Removes unsubscribed mods
- Built with [Rust](http://rust-lang.org)

You can see this project's backlog [here](https://github.com/users/valentinegb/projects/4).

## How to Install

### Windows

> [!NOTE]
> You may be prevented from launching the program by a popup warning you that the program is not safe. It is, and if you don't believe me you can check the code yourself. To launch the program anyway, click "More info" then "Run anyway".

Go to the [latest release on GitHub](https://github.com/valentinegb/bonelab_mod_manager/releases/latest) and download `bonelab_mod_manager.exe`. That's it, you can put that exectuable wherever you like.

### macOS

> [!NOTE]
> Unfortunately, I am not part of Apple's developer program so I am unable to sign the mod manager macOS app package. This means that when you first attempt to launch the app, macOS will stop you saying that the developer cannot be verified and that the application may contain malware. To circumvent this, you can right click the app in your Dock, click Options then Show in Finder, right click the app in finder, click Open, then in the popup click Open again. From there on you should not be prevented from launching the app normally again.

#### Homebrew (Recommended)

[Homebrew](https://brew.sh) is a package manager for macOS. Since Bonelab Mod Manager is not available on the App Store and it does contain a self-updater, this is the recommended installation method if you want easy updates.

Once Homebrew is installed, which can be done from the hyperlink above, to install Bonelab Mod Manager you simply need to enter the following into your terminal:

```shell
brew install valentinegb/tap/bonelab-mod-manager
```

To update later on, enter this command:

```shell
brew upgrade bonelab-mod-manager
```

#### Manual

Go to the [latest release on GitHub](https://github.com/valentinegb/bonelab_mod_manager/releases/latest) and download `bonelab_mod_manager.dmg`. When the disk image has downloaded, open it and drag the Bonelab Mod Manager app into the Applications alias. You will have to repeat this process to update.

### Linux

Go to the [latest release on GitHub](https://github.com/valentinegb/bonelab_mod_manager/releases/latest) and download `bonelab_mod_manager.tar.gz`. When the archive has downloaded, extract it. You can put the contained executable wherever you like.

## How to Use

1. Follow the installation instructions for your OS [above](#how-to-install).
2. Launch the program, by double clicking or another preferred method.
3. On your first time running, you will be prompted to sign in to mod.io. Use the arrow keys to select your method of signing in then press enter.
   - If you select "Send me an email code" you will need to type in the email associated with your mod.io account. You will shortly after receive an email from mod.io containing a code, which you will also need to enter into the program.
   - Selecting "Let me input my token" is a little more complicated. You will need to go to [mod.io Account Access](https://mod.io/me/access) and generate a token with read permissions. (You do **NOT** need to give write permissions and I advise that you don't.) You can name the client and the token whatever you'd like, it doesn't matter.
5. If you are on Windows, you will be prompted whether you would like to install PC mods or Quest mods.
6. After that, the program will do the rest for you! Sit back, relax, and watch as all your mods are downloaded and installed.
7. Next time you want to sync your installed mods, just run the program again.

## Quest Mod Installation Path

PC mods are installed directly into a PC Bonelab installation, but Quest mods must be manually put onto your headset. Quest mods are installed in a directory dependent on your OS.

macOS: `~/Library/Application Support/com.valentinegb.bonelab_mod_manager/Mods`

Linux: `~/var/lib/bonelab_mod_manager/Mods`

Windows: `%AppData%/bonelab_mod_manager/Mods`

I know, it isn't ideal having to move the mods from this folder to your headset,
but it's on the roadmap for the mod manager to do that for you, it will in the future!

## Configuring Concurrent Downloads

By default, Bonelab Mod Manager will install 4 mods at a time. This is configurable however since hardware can differ drastically. To change the maximum number of concurrent downloads, set the `BMM_CONCURRENT_DOWNLOADS` environment variable.
