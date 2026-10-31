---
title: CLI reference
description: Command-line usage and options for oxiviz.
---

:::caution[Planned interface]
oxiviz is in early development. This page describes the planned command-line interface, which may change before the
first release.
:::

```sh
oxiviz <INPUT> -o <OUTPUT> [OPTIONS]
```

## Arguments

| Argument | Description |
|---|---|
| `<INPUT>` | Audio file to visualize. Any format ffmpeg can read works, such as MP3, FLAC, WAV, Ogg, M4A or Opus. |

## Options

| Option | Default | Description |
|---|---|---|
| `-o`, `--output <FILE>` | (required) | Path of the video to write. The container format follows the file extension, e.g. `.mp4`. |
| `--size <WxH>` | `1280x720` | Output resolution in pixels. |
| `--fps <N>` | `30` | Output frame rate. |
| `--seed <N>` | random | Seed for all randomness. Use the same seed to get the same video. See [Determinism and seeds](/concepts/determinism/). |
| `-h`, `--help` | | Show help. |
| `-V`, `--version` | | Show the version. |

## Exit codes

| Code | Meaning |
|---|---|
| `0` | The video was rendered successfully. |
| non-zero | Something failed. The error is written to stderr. |

## Examples

```sh
# 1080p at 60 fps
oxiviz song.flac -o song.mp4 --size 1920x1080 --fps 60

# Vertical video for phones
oxiviz song.mp3 -o vertical.mp4 --size 1080x1920

# Reproducible render
oxiviz song.mp3 -o song.mp4 --seed 1234
```
