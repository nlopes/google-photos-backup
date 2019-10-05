use std::path::{Path, PathBuf};

use clap::value_t;
use regex::Regex;

use crate::filesystem::ensure_folder;

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub shared_albums: bool,
    pub album_filter: Option<Regex>,
    pub media_filter: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct Config {
    cache_dir: PathBuf,
    archive_dir: PathBuf,
    options: CliOptions,
}

impl Config {
    pub fn new<'a>(args: &clap::ArgMatches<'a>) -> Self {
        let archive_dir = PathBuf::from(args.value_of("BACKUP FOLDER").unwrap());
        let mut cache_dir = ::dirs::cache_dir().expect("Could not get cache dir");
        cache_dir.push("google_photos_backup");
        ensure_folder(&cache_dir);

        let shared_albums = args.is_present("shared_albums");
        let album_filter = value_t!(args, "album_filter", Regex).ok();
        let media_filter = value_t!(args, "media_filter", Regex).ok();

        Self {
            cache_dir,
            archive_dir: Config::discover_archive_fullpath(&archive_dir),
            options: CliOptions {
                shared_albums,
                album_filter,
                media_filter,
            },
        }
    }

    fn discover_archive_fullpath(basepath: &Path) -> PathBuf {
        match basepath.canonicalize() {
            Ok(path) => path,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    ensure_folder(basepath);
                    basepath.to_path_buf()
                }
                e => {
                    panic!("Can't deal with unknown path: {:?}", e);
                }
            },
        }
    }

    pub fn cache(&self) -> PathBuf {
        self.cache_dir.clone()
    }

    pub fn archive(&self) -> PathBuf {
        self.archive_dir.clone()
    }

    pub fn options(&self) -> CliOptions {
        self.options.clone()
    }
}
