---
title: Running in a container
description: Render videos with oxiviz on a server or in Docker.
---

oxiviz renders on the CPU and doesn't need a GPU or display, so it fits in a plain container. It only needs the
`oxiviz` binary and `ffmpeg`.

## Example Dockerfile

```dockerfile
FROM rust:1-slim AS build
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ffmpeg \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/oxiviz /usr/local/bin/oxiviz
ENTRYPOINT ["oxiviz"]
```

```sh
docker build -t oxiviz .
docker run --rm -v "$PWD:/work" -w /work oxiviz song.mp3 -o song.mp4
```

## Calling it from another program

Treat oxiviz like any other command-line tool: start it as a subprocess, wait for it to exit, and check the exit code.
A non-zero exit means the render failed, and the error is written to stderr.

## Performance

- Rendering uses all CPU cores. In a shared environment, limit it with the container's CPU quota.
- Memory use is small and doesn't grow with track length, because frames are streamed to ffmpeg instead of kept in
  memory.
