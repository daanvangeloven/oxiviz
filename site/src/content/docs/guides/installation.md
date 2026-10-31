---
title: Installation
description: Build and install oxiviz from source.
---

oxiviz is written in Rust and uses ffmpeg to read audio and write video.

## Requirements

- **Rust 1.85 or newer** (the project uses the 2024 edition). Install it with [rustup](https://rustup.rs/).
- **ffmpeg** on your `PATH`. oxiviz runs `ffmpeg` as a separate process, so any recent build works.

Check both:

```sh
cargo --version
ffmpeg -version
```

## Build from source

```sh
git clone <repository-url> oxiviz
cd oxiviz
cargo install --path crates/cli
```

This builds an optimized `oxiviz` binary and puts it in `~/.cargo/bin`.

To build without installing:

```sh
cargo build --release
# binary at target/release/oxiviz
```

:::tip
Always use release builds to render. Debug builds are slow enough that even short clips take a long time.
:::
