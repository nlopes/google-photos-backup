use log::info;
use regex::Regex;

use crate::filtering::Matcher;
use google_photoslibrary1::MediaItem;

impl Matcher for MediaItem {
    fn matches(&self, regex: Option<Regex>) -> bool {
        let mut filename_filtered = true;
        let mut description_filtered = true;
        if let Some(media_filter) = regex {
            if let Some(ref filename) = self.filename {
                if !media_filter.is_match(filename) {
                    filename_filtered = false;
                }
            } else {
                filename_filtered = false;
            }
            if let Some(ref description) = self.description {
                if !media_filter.is_match(description) {
                    description_filtered = false;
                }
            } else {
                description_filtered = false;
            }
        }

        if !filename_filtered && !description_filtered {
            info!("Skipping media due to filtering...");
        }
        filename_filtered || description_filtered
    }
}
