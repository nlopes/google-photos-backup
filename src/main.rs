use google_photoslibrary1::Result;
use tokio::prelude::future::lazy;

use google_photos_backup::cli::cli;
use google_photos_backup::config::Config;
use google_photos_backup::library::Library;
use google_photos_backup::worker;

fn main() -> Result<()> {
    env_logger::init();
    let args = cli();
    let config = Config::new(&args);
    let library = Library::new(&config);

    let mut runtime = worker::start();
    let (work_sender, work_receiver) = worker::channel();

    runtime.spawn(lazy(move || worker::process_work(work_receiver)));

    if config.options().shared_albums {
        library.download_shared_albums(&work_sender)?;
    }
    library.download_media_items(&work_sender)?;
    library.download_albums()
}
