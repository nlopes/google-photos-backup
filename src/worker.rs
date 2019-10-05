use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use google_photoslibrary1::MediaItem;
use log::{debug, error, info};
use reqwest;
use tokio::prelude::future::{lazy, Future, IntoFuture};
use tokio::runtime::{Builder, Runtime};
use tokio_timer::clock::Clock;

use crate::filesystem::FilesystemSafeEscaper;

const MAX_BATCH_SIZE: usize = 25;

pub fn channel() -> (Sender<(MediaItem, PathBuf)>, Receiver<(MediaItem, PathBuf)>) {
    unbounded()
}

pub fn start() -> Runtime {
    let mut builder = Builder::new();
    builder
        .blocking_threads(1)
        .core_threads(1)
        .clock(Clock::system())
        .keep_alive(Some(Duration::from_secs(60)))
        .name_prefix("work_processing_runtime")
        .stack_size(3 * 1024 * 1024)
        .build()
        .unwrap()
}

fn write_file(mut resp: reqwest::Response, filepath: &Path) {
    File::create(filepath)
        .and_then(|f| {
            let mut writer = std::io::BufWriter::new(f);
            match resp.copy_to(&mut writer) {
                Ok(written) => {
                    if let Some(size) = resp.content_length() {
                        assert_eq!(written, size);
                    }
                    debug!("Got file and saved it with {} bytes written", written)
                }
                Err(e) => error!("Could not write to file: {}", e),
            };
            Ok(())
        })
        .map_err(|e| error!("Could not create filepath {:?}: {}", filepath, e))
        .ok();
}

/*
In `get` we make a http request and save the body to a file. If there are any errors saving the file, we
log but don't take any recoverable action (yet).
*/
fn get(client: &reqwest::Client, url: &str, filepath: &Path) {
    let mut success = false;
    let mut retries = 5;
    let mut sleep_duration = 100;

    while !success && retries > 0 {
        client
            .get(&format!("{}=d", url))
            .send()
            .and_then(|resp| {
                match resp.status() {
                    reqwest::StatusCode::OK => {
                        // TODO(nlopes): need to check if write was successful
                        write_file(resp, filepath);
                        success = true; // wrong assumption that if I get here, I got the file
                    }
                    status => {
                        error!("Got unexpected status code: {:?}", status);
                    }
                };
                Ok(())
            })
            .map_err(|e| error!("Unable to download file: {}", e))
            .unwrap_or(());

        if !success {
            retries -= 1;
            std::thread::sleep(Duration::from_millis(sleep_duration));
            sleep_duration *= 2;
            debug!("Retrying file download, {} retries left.", retries);
        }
    }
}

pub fn process_work(
    receiver: Receiver<(MediaItem, PathBuf)>,
) -> impl Future<Item = (), Error = ()> {
    let mut builder = Builder::new();
    let mut runtime = builder
        .blocking_threads(400)
        .clock(Clock::system())
        .keep_alive(Some(Duration::from_secs(60)))
        .name_prefix("process-work")
        .stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();

    let mut batch = Vec::new();
    let mut expired = false;
    let client = reqwest::Client::new();

    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok((media, basepath)) => {
                if let Some(filename) = &media.filename {
                    let filepath = basepath.join(filename.escape());
                    if filepath.exists() {
                        debug!("File already exists, ignoring file {:?}", filepath);
                    } else {
                        debug!("Downloading {} to {:?}", filename, basepath);
                        batch.push((media, filepath));
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                debug!("Got timeout, expiring batch");
                expired = true;
            }
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        };

        let b = batch.clone();
        if b.len() > MAX_BATCH_SIZE || expired {
            for (media, filepath) in b {
                let client = client.clone();
                runtime.spawn(lazy(move || {
                    debug!("Downloading {:?}", filepath);
                    if let Some(url) = media.base_url {
                        get(&client, &url, &filepath);
                    }
                    Ok(()).into_future()
                }));
            }
            batch.clear();
            expired = false;
        }
    }

    info!("Finishing downloading media");
    runtime
        .shutdown_on_idle()
        .wait()
        .expect("unable to shutdown batch processing runtime");
    Ok(()).into_future()
}
