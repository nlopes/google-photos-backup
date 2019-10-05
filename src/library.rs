use std::path::PathBuf;

use crossbeam_channel::Sender;
use google_photoslibrary1::{
    Album, MediaItem, PhotosLibrary, Result, SearchMediaItemsRequest, SearchMediaItemsResponse,
};
use hyper::{client::Response, net::HttpsConnector, status::StatusCode, Client};
use hyper_rustls::TlsClient;
use log::{error, info};

use crate::album::AlbumFetcher;
use crate::auth::{authenticate, LibraryAuthenticator};
use crate::config::Config;
use crate::filtering::Matcher;

const ALBUM_LIST_MAX_PAGE_SIZE: i32 = 50;
const MEDIA_LIST_MAX_PAGE_SIZE: i32 = 100;

pub struct Library {
    hub: PhotosLibrary<Client, LibraryAuthenticator>,
    config: Config,
}

impl Library {
    pub fn new(config: &Config) -> Self {
        let auth = authenticate(config);
        let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
        let hub = PhotosLibrary::new(client, auth);

        // This triggers authentication if there isn't a token present.  (This is an
        // optimisation for user flow that only affects performance once! Doing this adds one
        // unnecessary http call but on the other hand it makes it possible for the user to
        // only deal with permissions once, at the beginning of the program).
        let req = google_photoslibrary1::SearchMediaItemsRequest {
            album_id: None,
            page_size: Some(1),
            page_token: None,
            filters: None,
        };
        let _ = hub.media_items().search(req).doit();
        let _ = hub.albums().list().page_size(1).doit();

        Self {
            hub,
            config: config.clone(),
        }
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn search_media_items(
        &self,
        search: SearchMediaItemsRequest,
    ) -> Result<(Response, SearchMediaItemsResponse)> {
        self.hub.media_items().search(search).doit()
    }

    fn get_albums(&self) -> Result<Vec<Album>> {
        info!("Getting albums metadata");
        let mut page_token = String::from("");
        let mut albums_found = Vec::new();

        loop {
            let mut builder = self
                .hub
                .albums()
                .list()
                .page_size(ALBUM_LIST_MAX_PAGE_SIZE)
                .exclude_non_app_created_data(false);

            if page_token != "" {
                builder = builder.page_token(&page_token);
            }

            let result = builder.doit();

            match result {
                Err(e) => return Err(e),
                Ok((http_response, albums_response)) => {
                    if http_response.status == StatusCode::Ok {
                        if let Some(albums) = albums_response.albums {
                            albums
                                .iter()
                                .filter(|album| album.matches(self.config().options().album_filter))
                                .for_each(|album| {
                                    albums_found.push(album.clone());
                                });
                        }
                        if let Some(token) = albums_response.next_page_token {
                            page_token = token;
                        } else {
                            info!("Found {} albums.", albums_found.len());
                            return Ok(albums_found);
                        }
                    }
                }
            };
        }
    }

    fn get_shared_albums(&self) -> Result<Vec<Album>> {
        info!("Getting shared albums metadata");
        let mut page_token = String::from("");
        let mut albums_found = Vec::new();

        loop {
            let mut builder = self
                .hub
                .shared_albums()
                .list()
                .page_size(ALBUM_LIST_MAX_PAGE_SIZE)
                .exclude_non_app_created_data(false);

            if page_token != "" {
                builder = builder.page_token(&page_token);
            }

            let result = builder.doit();

            match result {
                Err(e) => return Err(e),
                Ok((http_response, albums_response)) => {
                    if http_response.status == StatusCode::Ok {
                        if let Some(albums) = albums_response.shared_albums {
                            albums
                                .iter()
                                .filter(|album| album.matches(self.config().options().album_filter))
                                .for_each(|album| {
                                    albums_found.push(album.clone());
                                });
                        }
                        if let Some(token) = albums_response.next_page_token {
                            page_token = token;
                        } else {
                            info!("Found {} shared albums.", albums_found.len());
                            return Ok(albums_found);
                        }
                    }
                }
            };
        }
    }

    pub fn download_media_items(&self, work_sender: &Sender<(MediaItem, PathBuf)>) -> Result<()> {
        info!("Retrieving media...");
        let mut page_token = String::new();
        let mut count = 0;
        loop {
            let mut builder = self
                .hub
                .media_items()
                .list()
                .page_size(MEDIA_LIST_MAX_PAGE_SIZE);
            if page_token != "" {
                builder = builder.page_token(&page_token);
            }
            match builder.doit() {
                Err(e) => return Err(e),
                Ok((http_response, items_response)) => {
                    if http_response.status == StatusCode::Ok {
                        if let Some(items) = items_response.media_items {
                            count += items.len();
                            items
                                .iter()
                                .filter(|media| media.matches(self.config().options().media_filter))
                                .for_each(|media| {
                                    work_sender
                                        .send((media.clone(), self.config.archive()))
                                        .unwrap_or_else(|e| {
                                            error!("Error sending to be processed: {}", e)
                                        });
                                });
                        }
                        if let Some(token) = items_response.next_page_token {
                            page_token = token;
                        } else {
                            info!("Retrieved {} items", count);
                            return Ok(());
                        }
                    }
                }
            };
        }
    }

    pub fn download_albums(&self) -> Result<()> {
        let albums = self.get_albums()?;
        for album in albums {
            let album_path = album.create_dir(&self.config.archive()).unwrap();
            album
                .link_media_items(self, &self.config.archive(), &album_path)
                .unwrap();
        }
        Ok(())
    }

    pub fn download_shared_albums(&self, sender: &Sender<(MediaItem, PathBuf)>) -> Result<()> {
        let albums = self.get_shared_albums()?;
        for album in albums {
            let album_path = album.create_dir(&self.config.archive()).unwrap();
            album
                .download_media_items(self, &album_path, sender)
                .unwrap();
        }
        Ok(())
    }
}
