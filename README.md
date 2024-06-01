# Panther
Android GPS tracker

## Build

### Gradle build, precompiled rust library
If you want to get it working fast.
```bash
./run_gradle_precompiled.sh
```

### Full rust + gradle build

To get rustup toolchain installation script, visit https://rustup.rs/

After installing the toolchain, you need to install some additional tools:
```bash
cargo install cargo-ndk
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

To build project under android platform, `cargo ndk` is used. It uses standard installation path or environment variables 
ANDROID_NDK_HOME, ANDROID_HOME (You can change it in run_gradle.sh script). It is assumed that you are using Linux with 
standard installation of android studio. Also, you need to install NDK: Settings > Languages and frameworks > Android SDK > SDK Tools >
install NDK (side by side) > OK.

To launch full build Rust + Gradle:
```bash
./run_gradle.sh
```
