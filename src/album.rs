use std::fs;
use std::path::{Path, PathBuf};

use crossbeam_channel::Sender;
use google_photoslibrary1::{Album, MediaItem, Result as PLResult, SearchMediaItemsRequest};
use hyper::status::StatusCode;
use log::{debug, error, info};
use regex::Regex;

use crate::filesystem::{create_symlink, FilesystemSafeEscaper};
use crate::filtering::Matcher;
use crate::library::Library;

const MEDIA_SEARCH_MAX_PAGE_SIZE: i32 = 100;

impl Matcher for Album {
    fn matches(&self, regex: Option<Regex>) -> bool {
        if let Some(album_filter) = regex {
            if let Some(ref title) = self.title {
                if !album_filter.is_match(title) {
                    info!("Skipping album due to filtering ({})", title);
                    return false;
                }
            } else {
                info!("Skipping album due to filtering (no album title)");
                return false;
            }
        }
        true
    }
}

pub trait AlbumFetcher {
    fn safe_title(&self) -> String;
    fn create_dir(&self, basepath: &Path) -> Result<PathBuf, std::io::Error>;
    fn get_album_media(&self, library: &Library) -> PLResult<Vec<MediaItem>>;
    fn link_media_items(
        &self,
        library: &Library,
        basepath: &Path,
        album_path: &PathBuf,
    ) -> PLResult<()>;
    fn download_media_items(
        &self,
        library: &Library,
        album_path: &PathBuf,
        work_sender: &Sender<(MediaItem, PathBuf)>,
    ) -> PLResult<()>;
}

impl AlbumFetcher for Album {
    fn safe_title(&self) -> String {
        if let Some(title) = &self.title {
            title.escape()
        } else if let Some(id) = &self.id {
            info!(
                "Album has no title, creating album folder using id instead: {}.",
                id
            );
            id.to_string()
        } else {
            "_notitle-noid".to_string()
        }
    }

    fn create_dir(&self, basepath: &Path) -> Result<PathBuf, std::io::Error> {
        let title = self.safe_title();
        let path = basepath.join(title);
        debug!("Creating album folder: {}", &path.to_str().unwrap());
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn get_album_media(&self, library: &Library) -> PLResult<Vec<MediaItem>> {
        info!("Getting album media for album {}", self.safe_title());
        let mut media = Vec::new();
        if let Some(id) = &self.id {
            let mut page_token = None;
            loop {
                let req = SearchMediaItemsRequest {
                    album_id: Some(id.to_string()),
                    page_size: Some(MEDIA_SEARCH_MAX_PAGE_SIZE),
                    page_token: page_token.clone(),
                    filters: None,
                };
                let result = library.search_media_items(req);
                match result {
                    Err(e) => return Err(e),
                    Ok((http_response, items_response)) => {
                        if http_response.status == StatusCode::Ok {
                            if let Some(items) = items_response.media_items {
                                media.append(&mut items.clone());
                            }
                            if let Some(token) = items_response.next_page_token {
                                page_token = Some(token);
                            } else {
                                info!("Returning {} items from album", media.len());
                                return Ok(media);
                            }
                        }
                    }
                };
            }
        } else {
            info!("Returning {} items from album", media.len());
            Ok(media)
        }
    }

    fn link_media_items(
        &self,
        library: &Library,
        basepath: &Path,
        album_path: &PathBuf,
    ) -> PLResult<()> {
        let album_media = self.get_album_media(library)?;
        info!(
            "Linking album media items ({}) for album {}",
            album_media.len(),
            self.safe_title()
        );
        album_media
            .iter()
            .filter(|media| media.matches(library.config().options().media_filter))
            .for_each(|media| {
                if let Some(filename) = &media.filename {
                    let fname = filename.escape();
                    create_symlink(&basepath.join(&fname), &album_path.join(&fname));
                }
            });
        Ok(())
    }

    fn download_media_items(
        &self,
        library: &Library,
        album_path: &PathBuf,
        work_sender: &Sender<(MediaItem, PathBuf)>,
    ) -> PLResult<()> {
        let album_media = self.get_album_media(library)?;
        info!(
            "Downloading album media items ({}) for album {}",
            album_media.len(),
            self.safe_title()
        );
        album_media
            .iter()
            .filter(|media| media.matches(library.config().options().media_filter))
            .for_each(|media| {
                work_sender
                    .send((media.clone(), album_path.to_path_buf()))
                    .unwrap_or_else(|e| error!("Error sending to be processed: {}", e));
            });
        Ok(())
    }
}
