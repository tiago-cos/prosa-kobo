CREATE TABLE IF NOT EXISTS linked_devices (
    device_id TEXT PRIMARY KEY NOT NULL,
    lookup_key TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    client_device_id TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS cover_versions (
    device_id TEXT NOT NULL,
    book_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    PRIMARY KEY(device_id, book_id)
);

CREATE TABLE IF NOT EXISTS etags (
    book_id TEXT PRIMARY KEY NOT NULL,
    etag TEXT NOT NULL
);
