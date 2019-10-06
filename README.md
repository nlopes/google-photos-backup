# Google Photos Backup

[![Build Status](https://travis-ci.org/nlopes/google-photos-backup.svg?branch=master)](https://travis-ci.org/nlopes/google-photos-backup)
[![Build status](https://ci.appveyor.com/api/projects/status/57a7yxinpc5yas43/branch/master?svg=true)](https://ci.appveyor.com/project/nlopes/google-photos-backup/branch/master)
[![Apache 2.0 licensed](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://github.com/nlopes/google-photos-backup/blob/master/LICENSE)

Command line interface to backup your Google Photos.

## Build

In order to build `google-photos-backup` you'll have to generate a OAuth 2.0 client
credential per instructions at
[Authentication](https://cloud.google.com/docs/authentication/).

```bash
$ export GOOGLE_PHOTOS_BACKUP_CLIENT_ID=<client_id>
$ export GOOGLE_PHOTOS_BACKUP_CLIENT_SECRET=<client_secret>
$ cargo build --release
```

If you use one of the pre-built binaries, you don't need to worry as they already include
a `client_id` and `client_secret`.

## Usage

### First run

The first time you run `google-photos-backup`, you'll have to authenticate with your
Google account and authorize `google-photos-backup`. It will ask for permissions twice
(both permissions are needed for the read and listing operations on Albums and
MediaItems).

The second screen will ask you to copy&paste a code into the console.

The screens for authorisation will look like this:

<img src="https://raw.githubusercontent.com/nlopes/google-photos-backup/master/screenshots/view.png" width="30%" height="30%">
<img src="https://raw.githubusercontent.com/nlopes/google-photos-backup/master/screenshots/view-manage.png" width="30%" height="30%">

Once you authorise `google-photos-backup`, we store your tokens locally and use that
without asking again. If you ever need to re-authenticate, just delete the
credentials.json file and re-run `google-photos-backup`.

Running the program only requires you to provide the folder path where you want to download the media and albums to:

```bash
$ ./google-photos-backup BackupFolder
```

### Shared Albums

By default, `google-photos-backup` doesn't try to download shared albums. To include
shared albums in the downloads, use the `--shared-albums` flag.

When you download shared albums, we download the images directly into the folder, instead
of linking to images in the BackupFolder. I prefer that behaviour but if you have feedback
on this, please let me know!

```bash
$ ./google-photos-backup --shared-albums BackupFolder
```

### Filtering

For now, you can filter on an album title, and media filename and description. The flags, respectively are:
  - `--album-filter` - filter on album title
  - `--media-filter` - filter on media filename (or description)

Let's suppose you want to download only albums with "Vacation" in their title. You issue the following:

```bash
$ ./google-photos-backup --album-filter "Vacation" BackupFolder
```

The string after `--album-filter` is a regular expression, so you can do more complicated filtering:

```bash
$ ./google-photos-backup --album-filter "^Vacation.*2018$" BackupFolder
```

The above will filter for albums that start with "Vacation" and end with "2018".

If you want to download only media that has the mp4 extension and from albums with reef in the title, you can do the folllowing:

```bash
$ ./google-photos-backup --album-filter "reef" --media-filter "\.mp4$" BackupFolder
```

# License

This project is under the Apache License Version 2.0.
