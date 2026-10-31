---
title: Determinism and seeds
description: Why the same seed always renders the same video.
---

Everything random in oxiviz, including which effects play, their settings, colors and timing, comes from a single
random number generator that starts from a **seed**.

Render the same audio with the same seed and settings and you get the same video, frame for frame:

```sh
oxiviz song.mp3 -o a.mp4 --seed 42
oxiviz song.mp3 -o b.mp4 --seed 42   # same frames as a.mp4
```

## What affects the output

A render is reproducible as long as all of these stay the same:

- the audio
- the seed
- the output size and frame rate
- the oxiviz version

Changing any of them can change the visuals. A different size, for example, changes where effects are positioned, and a
different frame rate changes how the audio is sliced. Newer versions of oxiviz may also change how effects behave.

## Why it matters

- **Favorite renders:** if a render looks great, write down its seed and you can render it again at any time, for
  example at a higher resolution later.
- **Caching and pipelines:** an automated system can treat the audio, seed and settings as a cache key.
- **Bug reports:** a seed plus the audio is enough to reproduce a problem exactly.
