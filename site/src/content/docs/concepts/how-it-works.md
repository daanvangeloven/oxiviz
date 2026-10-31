---
title: How it works
description: The rendering pipeline, from audio to video frames.
---

oxiviz renders a video one frame at a time. Each frame goes through three stages: audio analysis, scheduling and
rendering.

```
audio file ─▶ ffmpeg (decode) ─▶ analysis ─▶ scheduler ─▶ effects ─▶ ffmpeg (encode) ─▶ video
```

## 1. Audio analysis

ffmpeg decodes the audio. oxiviz then cuts it into one slice per video frame and computes, for each slice:

- a **spectrum**: how loud each frequency is
- a **waveform**: the raw shape of the sound, which some effects draw as lines
- a **level**: the loudness of the lowest bass frequencies, which drives most of the motion
- a **treble** value: the loudness of the high frequencies, used for some effects such as the spin of Polygram

### Beat detection

oxiviz keeps a rolling average of the level over the last 30 frames:

- A **beat** fires when the level is more than 10% above that average, at most once every 10 frames.
- A **big beat** fires when, in addition, the level is high in absolute terms and more than 20% above the average,
  at most once every 20 frames.

Effects respond to beats in different ways. For example, Twin Orbs swell on every beat, and Surge zooms in on big
beats.

## 2. Scheduling

The visuals change over time because a scheduler keeps a small, changing set of effects running:

- exactly one **feedback warp** (Undertow), which moves the whole image
- one or more **overlays**, which draw shapes on top
- sometimes a **zoom** (Surge)

Each slot runs its effect for a random number of frames, roughly 2 to 15 seconds at 30 fps. When an
effect's time is up, it fades out and the scheduler replaces it with a randomly chosen effect with fresh, randomized
settings. Every one to four minutes the whole cycle restarts and every slot is picked again.

Warps don't switch abruptly either: a new warp blends in from the old one over a series of steps, so the motion morphs
from one pattern into the next.

## 3. Rendering

Each frame is built in this order:

1. **Warp:** every pixel of the new frame is copied from a displaced position in the previous frame, then softly
   blurred. This feedback loop creates the trails, smears and spirals.
2. **Overlays:** shapes are drawn directly onto the warped frame. On the next frame they get warped too, which is why
   every shape leaves a trail.
3. **View:** if Surge is active, the output is a zoomed-in crop of the frame.

Rendering is parallelized across CPU cores, and finished frames are streamed straight to ffmpeg.

## Learn more

- [Effects](/concepts/effects/): what each warp and overlay does
- [Determinism and seeds](/concepts/determinism/): why the same seed gives the same video
