---
title: Introduction
description: What oxiviz is and what it's for.
---

oxiviz is an offline music visualizer. You give it an audio file and it renders a video in which the visuals react to
the music. Its look is inspired by **Alchemy**, the classic Windows Media Player visualization: a feedback warp that
smears and swirls the previous frame, with beat-reactive shapes drawn on top.

:::caution[Early development]
oxiviz is still being built. The commands and options in these docs describe the planned interface and may change
before the first release.
:::

## What it's good for

- Giving audio-only tracks a music video for platforms that need video.
- Background visuals for streams, listening parties or displays.
- Batch rendering on a server, since it needs no GPU or display.

## What it isn't

oxiviz isn't a real-time visualizer for live audio input. It renders files offline, frame by frame, as fast as your CPU
allows. Rendering can be faster or slower than real time, depending on resolution and core count.

## Next steps

- [Install oxiviz](/guides/installation/)
- [Render your first video](/guides/quick-start/)
- [Learn how it works](/concepts/how-it-works/)
