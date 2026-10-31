---
title: Quick start
description: Render your first visualizer video.
---

Render a song to video:

```sh
oxiviz song.mp3 -o song.mp4
```

oxiviz decodes the audio, renders one frame per video frame, and encodes the result together with the original audio
into `song.mp4`.

## Pick the size and frame rate

```sh
oxiviz song.flac -o song.mp4 --size 1920x1080 --fps 60
```

Larger sizes and higher frame rates look smoother but take longer: render time grows roughly with
`width × height × fps`.

## Get the same video again

Each render is driven by a seed. Pass `--seed` to make a render reproducible:

```sh
oxiviz song.mp3 -o take1.mp4 --seed 42
oxiviz song.mp3 -o take2.mp4 --seed 42   # identical to take1.mp4
oxiviz song.mp3 -o other.mp4 --seed 7    # different visuals, same song
```

If you like how a render turned out, keep its seed. See [Determinism and seeds](/concepts/determinism/).

## Next

- [All CLI options](/reference/cli/)
- [Running in a container](/guides/containers/)
