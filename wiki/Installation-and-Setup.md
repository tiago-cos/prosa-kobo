# Installation and Setup

Prosa-Kobo needs a running [Prosa](https://github.com/tiago-cos/prosa) to talk
to. It runs either as a Docker container, which is the recommended way, or as a
standalone binary.

## Docker

This `docker-compose.yml` runs both:

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

Then `docker compose up -d`. Prosa is available on port `5000`, and Prosa-Kobo,
which the Kobo talks to, on port `5001`.

Three things to know about this image:

- **Set Prosa's `AUTH__ADMIN_KEY` to something secret.** Prosa will not start
  without one at least 8 characters long. See
  [Prosa's installation guide](https://github.com/tiago-cos/prosa/wiki/Installation-and-Setup).
- **Persistence needs a named volume mounted at `/app/persistence`.** Bind
  mounts do not work, because the container runs rootless and cannot take
  ownership of a host directory.
- **`DATABASE__FILE_PATH` is ignored here.** The container unsets it on startup
  so the database stays inside the volume. Every other setting works normally;
  see [Configuration](Configuration).

## Binary

Build it, or take a release binary, then:

1. Make sure Prosa is running.
2. Write a configuration file if the defaults do not suit you. Start from
   [`src/config/example.toml`](https://github.com/tiago-cos/prosa-kobo/blob/master/src/config/example.toml).
   Prosa-Kobo looks for `src/config/configuration.toml`, relative to the
   directory it is started from, unless the `CONFIGURATION` environment
   variable says otherwise.
3. Run `prosa-kobo`.

The database lands at `persistence/database.db` unless configured otherwise.
Its directory is created on first run.

## Checking it came up

On startup, Prosa-Kobo waits for Prosa, retrying every five seconds, then
checks Prosa's version. It refuses to start against a Prosa whose major or minor
version differs from the one it was written for. Once both are up, the log
shows:

```
[2026-10-02 11:54:08]  INFO Connected to prosa 0.2.0 at http://127.0.0.1:5000
[2026-10-02 11:54:08]  INFO Middleware started on http://0.0.0.0:5001
```

`/health` answers with no credentials:

```bash
curl http://localhost:5001/health
```

```json
{ "status": "ok", "software": "prosa-kobo", "version": "0.1.0" }
```

Once it answers, carry on to [Connecting a Kobo](Connecting-a-Kobo).

## Behind a reverse proxy

Prosa-Kobo hands the Kobo URLs to come back to: the endpoint it links with, and
the addresses of books and covers. By default they are built from the host and
port the Kobo addressed, over `http`. Behind a reverse proxy the Kobo's scheme
and port are the proxy's, so set `[server.public]` to the address the Kobo
reaches. See [Configuration](Configuration).

## Upgrading

Prosa-Kobo's schema is managed with versioned, reversible migrations embedded
in the binary. On startup it applies whatever the database has not seen yet, so
replacing the binary or pulling a newer image is the whole of an upgrade.

A snapshot of the database is written next to it before anything is applied,
unless `backup_before_migration` is off. Snapshots are never cleaned up
automatically.

To see where a database stands:

```bash
prosa-kobo --migrate-status
```

In Docker:

```bash
docker exec prosa-kobo prosa-kobo --migrate-status
```

## Downgrading

```bash
prosa-kobo --migrate-down <version>
```

This reverts every migration newer than `<version>`.

**A binary can only revert migrations whose down scripts it carries.** So to
move from a newer Prosa-Kobo to an older one, run `--migrate-down` with the
*newer* binary first, with the server stopped, then swap in the older one.

In Docker, stop the container and run the newer image against the volume:

```bash
docker run --rm --entrypoint prosa-kobo -v prosa_kobo:/app/persistence tsousa28/prosa-kobo --migrate-down <version>
```
