# Mnem

Entity that lives, remembers and breathes

## Build

### Interface

- Pre requisites

```sh
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu

# 32 bit
rustup target add armv7-unknown-linux-gnueabihf
# -- or --
# 64 bit
rustup target add aarch64-unknown-linux-gnu
```

- Build

```sh
cargo build --target aarch64-unknown-linux-gnu
```
