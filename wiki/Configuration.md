# Configuration

Prosa-Kobo reads its settings from three places, each overriding the one before:

1. **Built-in defaults**, compiled into the binary.
2. **A TOML file**, `src/config/configuration.toml` by default.
3. **Environment variables.**

So you only have to write down what you want to change. Every setting has a
usable default; the one you are most likely to need is where Prosa is.

## The configuration file

Prosa-Kobo looks for `src/config/configuration.toml`, relative to the directory
it is started from, and runs on its defaults if there is none. Point `CONFIGURATION` at another path
to move it:

```bash
export CONFIGURATION=/etc/prosa-kobo/config.toml
```

The file need only contain what you are overriding. Here is every setting, at
its default value:

```toml
[server.bind]
host = "0.0.0.0"
port = 5001

# No default: set it behind a reverse proxy
# [server.public]
# scheme = "https"
# host = "books.example.com"
# port = 443

[database]
file_path = "persistence/database.db"
max_connections = 16
busy_timeout_seconds = 10
backup_before_migration = true

[prosa]
scheme = "http"
host = "127.0.0.1"
port = 5000

[kepub]
cache_size_mb = 256
```

## Environment variables

Every setting can also be given as an environment variable, named after its
section and key joined by **two** underscores. A nested section such as
`[server.bind]` takes one pair per level:

```bash
export SERVER__BIND__PORT=8080
export PROSA__HOST=prosa
```

Underscores *inside* a name are not separators and stay as they are, which is
why `[database].file_path` is `DATABASE__FILE_PATH`.

## Reference

### `[server.bind]`

| Setting | Meaning |
|---|---|
| `host` | Network interface to listen on. |
| `port` | Port to listen on. |

### `[server.public]`

The address the Kobo reaches Prosa-Kobo at. Every URL handed to the Kobo, the
linked endpoint included, is built from it.

Leave it out unless Prosa-Kobo sits behind a reverse proxy. Without it, those
URLs use `http` and the host and port the Kobo addressed, which is right when
the Kobo talks to Prosa-Kobo directly. Behind a proxy, the Kobo's scheme and
port are the proxy's.

| Setting | Meaning |
|---|---|
| `scheme` | `http` or `https`. |
| `host` | Hostname or IP address the Kobo uses. |
| `port` | Port the Kobo uses. |

All three are required once the section is present.

### `[database]`

| Setting | Meaning |
|---|---|
| `file_path` | Path to the SQLite database file. |
| `max_connections` | Size of the connection pool. |
| `busy_timeout_seconds` | How long a statement waits for a contended write before giving up. |
| `backup_before_migration` | Snapshot the database before applying or reverting migrations. Leave this on. |

### `[prosa]`

Where Prosa is.

| Setting | Meaning |
|---|---|
| `scheme` | `http` or `https`. |
| `host` | Hostname or IP address of the Prosa server. |
| `port` | Port of the Prosa server. |

### `[kepub]`

Prosa stores EPUBs, and the Kobo reads KEPUBs, so books are converted on the
way to the Kobo. Converted books are kept in memory, because conversion is slow,
and positions and highlights are translated against the converted book.

| Setting | Meaning |
|---|---|
| `cache_size_mb` | How much memory converted books may take, in megabytes. The least recently used are dropped first. |

## Logging

Log level comes from the standard `RUST_LOG` variable, and defaults to `info`.
Accepted values are `error`, `warn`, `info`, `debug` and `trace`:

```bash
export RUST_LOG=warn
```

At `debug`, requests Prosa-Kobo answers without handling them are logged too,
such as the Kobo asking for store recommendations.

Log lines never include the secret part of a device's endpoint.
