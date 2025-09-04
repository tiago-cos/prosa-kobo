# Prosa-Kobo

**A middleware service that bridges Kobo eReaders with [Prosa](https://github.com/tiago-cos/prosa).**

## Overview

Prosa-Kobo is a companion service to [Prosa](https://github.com/tiago-cos/prosa), written in Rust.

It acts as a translation layer between Kobo eReaders and the Prosa API, allowing Kobo devices to sync books, metadata, reading progress, and shelves.

Prosa-Kobo ensures that Kobo devices see Prosa as if it were the official Kobo backend, making it possible to manage your entire eBook collection on Kobo hardware.

Full documentation (including API docs): [tiago-cos.github.io/prosa-kobo](https://tiago-cos.github.io/prosa-kobo)

## Why Prosa-Kobo?

Kobo eReaders expect to communicate with Kobo’s cloud infrastructure. Prosa-Kobo emulates that behavior and forwards requests to Prosa, translating them as necessary. This makes it possible to fully integrate Kobo devices into your self-hosted Prosa ecosystem.

With Prosa-Kobo, you get:

* Full Kobo synchronization support
* Native experience on Kobo eReaders without modifying device firmware

## Features

* Bridges Kobo eReaders to Prosa
* Synchronization of books, metadata, shelves, and reading progress
* Supports multiple users and devices

## Build Instructions

```bash
git clone https://github.com/tiago-cos/prosa-kobo.git
cd prosa-kobo
cargo build --release
```

## Test Instructions

1. Clone the repository:

   ```bash
   git clone https://github.com/tiago-cos/prosa-kobo.git
   cd prosa-kobo/tests
   ```

2. Create a `.env.local` file in the `config` subfolder and configure the `MIDDLEWARE_URL` and `PROSA_URL` env variables (see `.env` in the same folder).

3. Make sure both **Prosa** and **Prosa-Kobo** are running.

   * Prosa requires `AUTH__ADMIN_KEY` to be set, for example:

     ```bash
     AUTH__ADMIN_KEY=admin_key ./prosa
     ```

   * Then run Prosa-Kobo:

     ```bash
     ./prosa-kobo
     ```

4. Run the tests:

   ```bash
   npm run test
   ```

## Roadmap

* [x] **Backend ([Prosa](https://github.com/tiago-cos/prosa))**
  * [x] **Books**
    * [x] File management
    * [x] Covers
    * [x] Metadata
    * [x] Annotations
    * [x] Reading progress
    * [x] Ratings
    * [ ] Reading time statistics
  * [x] **Shelves** (collections of books)
  * [x] **Users**
    * [x] Profiles
    * [x] Preferences
    * [x] API keys
  * [x] Automatic metadata retrieval
  * [x] Synchronization across devices
  * [ ] Audiobook support

* [x] **Kobo Support (Prosa-Kobo)**
  * [x] **Books**
    * [x] File management
    * [x] Covers
    * [x] Metadata
    * [x] Annotations
    * [x] Reading progress
    * [x] Ratings
    * [ ] Reading time statistics
  * [x] **Shelves**
  * [x] Prosa synchronization
  * [ ] Audiobooks

* [ ] **Mobile App**

  * TODO

## Related Projects

* [Prosa](https://github.com/tiago-cos/prosa) – the main backend and API for managing your eBook collection.
