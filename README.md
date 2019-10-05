# Google Photos Backup

Command line interface to backup your Google Photos.

## Build

In order to build `google-photos-backup` you'll have to generate a OAuth 2.0 client
credential per instructions at
[Authentication](https://cloud.google.com/docs/authentication/).

```bash
GOOGLE_PHOTOS_BACKUP_CLIENT_ID=<client_id> GOOGLE_PHOTOS_BACKUP_CLIENT_SECRET=<client_secret> cargo build --release
```

or

```bash
export GOOGLE_PHOTOS_BACKUP_CLIENT_ID=<client_id>
export GOOGLE_PHOTOS_BACKUP_CLIENT_SECRET=<client_secret>
cargo build --release
```

If you use one of the pre-built binaries, you don't need to worry as they already include
a `client_id` and `client_secret`.

## Running

The first time you run `google-photos-backup`, you'll have to authenticate with your
Google account and authorize `google-photos-backup`. It will ask for permissions twice
(both permissions are needed for the read and listing operations on Albums and
MediaItems).

The second screen will ask you to copy&paste a code into the console.

The screens for authorisation will look like this:

![View](https://raw.githubusercontent.com/nlopes/google-photos-backup/master/screenshots/view.png)

![View and Manage](https://raw.githubusercontent.com/nlopes/google-photos-backup/master/screenshots/view-manage.png)


Once you authorise `google-photos-backup`, we store your tokens locally and use that
without asking again. If you ever need to re-authenticate, just delete the
credentials.json file and re-run `google-photos-backup`.

## Filtering

TODO

# License

This project is under the Apache License Version 2.0.
