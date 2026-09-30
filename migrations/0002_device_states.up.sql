CREATE TABLE device_states (
    device_id TEXT NOT NULL REFERENCES linked_devices(device_id) ON DELETE CASCADE,
    book_id TEXT NOT NULL,
    status TEXT NOT NULL,
    chapter TEXT,
    span TEXT,
    PRIMARY KEY(device_id, book_id)
);
