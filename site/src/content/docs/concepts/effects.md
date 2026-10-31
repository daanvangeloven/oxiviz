---
title: Effects
description: The warps and overlays oxiviz combines to build its visuals.
---

oxiviz builds its visuals from a small set of effects. You don't choose them yourself. The
[scheduler](/concepts/how-it-works/#2-scheduling) picks and randomizes them as the video plays, so these descriptions
cover what you'll see.

## Undertow: the feedback warp

Undertow is always running and is the core of the look. Each frame it moves every pixel of the previous frame according
to a **warp map**, then blurs it slightly. Shapes drawn on top get pulled along, which leaves trails behind them.

Sometimes Undertow combines two warps, averaging their movement. When it switches warps, the new one blends in gradually.

### Weave

Narrow bands that slide in opposite directions, like threads in a weave. Band width and speed are randomized, and the
bands can run horizontally or vertically.

### Swirl

Rotates the image around a center point, either the middle of the screen or a random spot, while pulling pixels inward.
Some variants rotate in waves, so rings around the center turn in alternating directions.

### Scroll

Slides the image in a direction. Variants stretch it away from an edge or corner, or add a sine-wave ripple across it.

### Kaleidoscope

Applies symmetry first, then runs one of the other warps (or a plain rotation) inside the mirrored region. Symmetry
modes:

- **Quad mirror:** four mirrored quadrants.
- **Diagonal mirror:** folded along the screen diagonals.
- **Ring:** mirrored around a circle.
- **Tiles:** the area outside a central circle is tiled with small copies of the picture.
- **Checker:** only every other square of a checkerboard is warped.

## Overlays

Overlays draw shapes on top of the warped image. One to a few run at a time.

### Ribbon

A waveform line drawn through the center of the screen that spins over time. It can be mirrored, can stretch with the
bass, and sometimes gets a second line crossing it. Its color fades between two randomly chosen colors.

### Polygram

A polygon made of waveform lines, with anywhere from 3 to 20 points. Its size follows the bass, and the treble makes it
spin.

### Twin Orbs

Two glowing balls with a colored rim that drift and bounce off the edges, mirrored through the center of the screen.
They wobble like springs with the music, double in size on beats, and briefly connect with a waveform line.

## Surge

Surge is a zoom effect. On a big beat it jumps in toward the center, then eases back out with a gentle bobbing
motion until the next big beat.
