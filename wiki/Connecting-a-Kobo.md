# Connecting a Kobo

This walks through linking a Kobo to a Prosa account. It assumes Prosa and
Prosa-Kobo are both running, with Prosa on `http://localhost:5000` and
Prosa-Kobo on `http://192.168.1.20:5001`, an address the Kobo can reach over
your network. See [Installation and Setup](Installation-and-Setup) if they are
not.

Every endpoint is described in full in the
[API reference](https://tiago-cos.github.io/prosa-kobo), and Prosa's in
[Prosa's](https://tiago-cos.github.io/prosa).

## 1. Create an API key for the device

The Kobo acts in Prosa with an API key of its own. Sign in to Prosa to get a
JWT, then create one:

```bash
curl -X POST http://localhost:5000/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username": "me", "password": "a-good-password"}'

curl -X POST http://localhost:5000/users/$USER_ID/keys \
  -H "Authorization: Bearer $JWT" \
  -H 'Content-Type: application/json' \
  -d '{"name": "Bedside Kobo", "capabilities": ["Read", "Create", "Update", "Delete"]}'
```

The key needs `Read` to be linked at all. The Kobo also changes your library:
it saves reading progress, highlights and collections, and a book removed from
My Books is deleted from Prosa. Give it all four capabilities unless you want to
keep it from some of that.

Leave `expires_at` out, unless you want the Kobo to stop syncing on a given
date.

## 2. Link the device

```bash
curl -X POST http://192.168.1.20:5001/devices/linked \
  -H "Authorization: Bearer $JWT" \
  -H 'Content-Type: application/json' \
  -d '{"name": "Bedside Kobo", "api_key": "<the key from step 1>"}'
```

```json
{
  "device_id": "8Nn3sBOoRdSK5xUlpEDdxA",
  "api_endpoint": "http://192.168.1.20:5001/hT3kQ0vZ9mWpYc8LrNfJ2dGxAeBu6SsV1oIiKlMn"
}
```

The key is checked against Prosa before anything is stored, so a mistyped or
expired one is refused here rather than at the first sync.

Send the request to the address the Kobo will use. Unless `[server.public]` is
set, `api_endpoint` is built from it.

**`api_endpoint` is the device's credential.** Anyone who has it can reach the
library it is linked to. It is returned only here and never listed again.

## 3. Point the Kobo at Prosa-Kobo

Connect the Kobo to a computer over USB and open `.kobo/Kobo/Kobo eReader.conf`
on it. Under `[OneStoreServices]`, set:

```ini
api_endpoint=http://192.168.1.20:5001/hT3kQ0vZ9mWpYc8LrNfJ2dGxAeBu6SsV1oIiKlMn
```

Keep a copy of the line you replace, to point the Kobo back at Kobo's own
servers later. Nothing else in the file needs changing: the Kobo learns every
other address from Prosa-Kobo on its next sync.

Eject the Kobo and sync. Your books appear in the Books tab, and each downloads
the first time you open it.

## Managing devices

`GET /devices/linked?user_id=<your user id>` lists your linked devices, and
`DELETE /devices/linked/{device_id}` unlinks one. Its endpoint stops working at
once.

Each link is a device of its own, with an endpoint of its own. Linking the same
Kobo again gives it a new endpoint without retiring the old one, so update
`eReader.conf`, then unlink the device the old endpoint belonged to.

## What to expect on the Kobo

Most of the sync behaves as it would with Kobo's own servers. A few things are
worth knowing:

- **A position changed elsewhere is offered, not forced.** When a book's
  position changes in Prosa, the Kobo asks whether to go there the next time
  you open the book. It does not move on sync.
- **Except for a book the Kobo has as unread or finished.** The Kobo never
  asks for the state of such a book, so its new position arrives with the sync
  instead, and the book opens there without asking. Until it is opened, it
  shows 1% read.
- **Marking a book finished or unread forgets its position**, on the Kobo and in
  Prosa alike.
- **The publisher shown is the book file's own.** The Kobo reads it from the
  file rather than from the details it is sent, so a publisher changed in Prosa
  does not show.
- **A changed cover appears when it comes back into view.** Switch to another
  tab and back to see it.
- **The Collections tab does not refresh after a sync.** If it was open while
  you synced, switch to another tab and back.
- **A book with no title** is shown as "Untitled".
- **The Kobo cannot draw emoji**, and counts each as two characters, so a
  highlight near one may be offset.
