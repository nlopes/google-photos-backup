use clap::{clap_app, crate_version, crate_authors};

pub fn cli<'a>() -> clap::ArgMatches<'a> {
    clap_app!(
        google_photos_backup =>
            (version: crate_version!())
            (author: crate_authors!())
            (about: "Command line interface to backup your Google Photos")
            (@arg ("BACKUP FOLDER"): +required "Full path to the destination of the backup folder")
            (@arg shared_albums: --("shared-albums") "Include shared albums when downloading")
            (@arg album_filter: -a --("album-filter") +takes_value "Album title filter")
            (@arg media_filter: -m --("media-filter") +takes_value "Media filename/description filter")
    ).get_matches()
}
