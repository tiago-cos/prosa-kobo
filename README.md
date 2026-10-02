# Prosa-Kobo

**A middleware service that connects Kobo eReaders to [Prosa](https://github.com/tiago-cos/prosa).**

## Overview

Prosa-Kobo is a companion service to [Prosa](https://github.com/tiago-cos/prosa),
written in Rust. A Kobo is pointed at it instead of Kobo's own servers.
Prosa-Kobo answers the Kobo the way it expects, and translates each request into
calls to Prosa, so your self-hosted library syncs to the Kobo as if it came from
Kobo itself.

Nothing is forwarded to Kobo's servers, and nothing on the device is modified
apart from one line in its configuration file.

## Documentation

- **[Wiki](https://github.com/tiago-cos/prosa-kobo/wiki)**: installing,
  configuring and connecting a Kobo, and working on Prosa-Kobo itself
- **[API reference](https://tiago-cos.github.io/prosa-kobo)**: the health
  check and the endpoints that link and unlink devices

## Features

- Books, converted to KEPUB on the way to the Kobo

- Book details and covers, kept up to date as they change in Prosa

- Reading position, reading status and ratings, in both directions

- Highlights and notes, in both directions, down to the character

- Collections, synced with Prosa shelves in both directions

- Several devices and several users on one server

## Quick Start

```yaml
services:
  prosa-kobo:
    image: tsousa28/prosa-kobo
    container_name: prosa-kobo
    ports:
      - "5001:5001"
    environment:
      - PROSA__HOST=prosa
      - PROSA__PORT=5000
    volumes:
      - prosa_kobo:/app/persistence
    restart: unless-stopped

  prosa:
    image: tsousa28/prosa
    container_name: prosa
    ports:
      - "5000:5000"
    environment:
      - AUTH__ADMIN_KEY=very_secret_key
    volumes:
      - prosa_library:/app/library
    restart: unless-stopped

volumes:
  prosa_kobo:
  prosa_library:
```

`docker compose up -d`, then link your Kobo and point it at the endpoint you are
given, as described in
[Connecting a Kobo](https://github.com/tiago-cos/prosa-kobo/wiki/Connecting-a-Kobo).

Persistence needs **named volumes**; bind mounts do not work, because the
containers run rootless. The full story, including running Prosa-Kobo as a
binary, is in
[Installation and Setup](https://github.com/tiago-cos/prosa-kobo/wiki/Installation-and-Setup).

## Build Instructions

```bash
git clone https://github.com/tiago-cos/prosa-kobo.git
cd prosa-kobo
cargo build --release
```

To run it from source, start Prosa, then `cargo run`. Settings go in
`src/config/configuration.toml`; copy `src/config/example.toml` to start from
it. See
[Configuration](https://github.com/tiago-cos/prosa-kobo/wiki/Configuration).

## Test Instructions

```bash
cargo test
```

The tests use an in-memory stand-in for Prosa, so nothing else needs to be
running. Checking that stand-in against a real Prosa, and the semi-automated
test on a real Kobo, are described in
[Contributing and Architecture](https://github.com/tiago-cos/prosa-kobo/wiki/Contributing-and-Architecture).

## Roadmap

- [x] **Backend ([Prosa](https://github.com/tiago-cos/prosa))**
  - [x] **Books**
    - [x] File management
    - [x] Covers
    - [x] Metadata
    - [x] Annotations
    - [x] Reading progress
    - [x] Ratings
    - [ ] Reading time statistics
  - [x] **Shelves** (collections of books)
  - [x] **Users**
    - [x] Profiles
    - [x] Preferences
    - [x] API keys
  - [x] Automatic metadata retrieval
  - [x] Synchronization across devices
  - [ ] Audiobook support

- [x] **Kobo Support (Prosa-Kobo)**
  - [x] **Books**
    - [x] File management
    - [x] Covers
    - [x] Metadata
    - [x] Annotations
    - [x] Reading progress
    - [x] Ratings
    - [ ] Reading time statistics
  - [x] **Shelves**
  - [x] Prosa synchronization
  - [ ] Audiobooks

- [ ] **Mobile App**

  - TODO

## Contributing

Issues and pull requests are both welcome. See
[CONTRIBUTING.md](.github/CONTRIBUTING.md), and
[Contributing and Architecture](https://github.com/tiago-cos/prosa-kobo/wiki/Contributing-and-Architecture)
for how the code is laid out.

## Related Projects

- [Prosa](https://github.com/tiago-cos/prosa) – the main backend and API for managing your eBook collection.

## License

[MIT](LICENSE)
