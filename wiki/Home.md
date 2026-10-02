# Prosa-Kobo

Prosa-Kobo connects Kobo eReaders to [Prosa](https://github.com/tiago-cos/prosa),
a self-hosted book server. The Kobo is pointed at Prosa-Kobo instead of Kobo's
own servers. Prosa-Kobo answers it the way the Kobo expects, and translates each
request into calls to Prosa.

Nothing is forwarded to Kobo's servers, and nothing on the device is modified
apart from one line in its configuration file.

## Documentation

**For running a server**

- **[Installation and Setup](Installation-and-Setup)**: getting Prosa-Kobo
  running beside Prosa, with Docker or as a binary, and upgrading it afterwards
- **[Configuration](Configuration)**: every setting, where to put it, and how
  the layers override each other
- **[Connecting a Kobo](Connecting-a-Kobo)**: linking a device, pointing it at
  Prosa-Kobo, and what to expect once it syncs

**For working on Prosa-Kobo**

- **[Contributing and Architecture](Contributing-and-Architecture)**: how the
  code is laid out, how to build and test it, including on a real Kobo, and
  what a pull request needs

**API reference**

Prosa-Kobo's own API is the handful of endpoints that link and unlink devices.
Its reference is generated from the OpenAPI spec and published at
[tiago-cos.github.io/prosa-kobo](https://tiago-cos.github.io/prosa-kobo).

## What syncs

- **Books.** Books added to Prosa appear on the Kobo, converted to KEPUB on the
  way. Books replaced or deleted in Prosa are replaced or removed. A book
  removed from My Books on the Kobo is deleted from Prosa.
- **Details and covers.** Title, authors, series, description, publication date
  and ISBN, and the cover, kept up to date as they change in Prosa.
- **Reading.** Reading position, reading status and rating, in both directions.
- **Annotations.** Highlights and notes, in both directions, down to the
  character.
- **Collections.** Kobo collections and Prosa shelves are the same thing, in
  both directions.

Several devices, and several users, can be linked to one Prosa-Kobo. Each
device acts with a Prosa API key of its own.
