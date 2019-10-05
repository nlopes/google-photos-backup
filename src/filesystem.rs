use std::path::Path;

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::symlink;

// In Windows I opted to go for a hard link. The reason being that software in Windows
// very often (if not always) doesn't process soft links as if it were a file: a good
// example is the Windows 10 Photos app, which seems to ignore soft links completely.
#[cfg(target_os = "windows")]
use std::fs::hard_link as symlink;

use log::debug;

pub fn ensure_folder(path: &Path) {
    std::fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("Could not create folder '{:?}': {}", path, e));
}

pub fn create_symlink(src: &Path, dst: &Path) {
    match symlink(src, dst) {
        Err(e) => debug!("Could not point {:?} to {:?}: {}", dst, src, e),
        Ok(()) => (),
    };
}

pub trait FilesystemSafeEscaper {
    fn escape(&self) -> String;
}

impl FilesystemSafeEscaper for String {
    fn escape(&self) -> String {
        if cfg!(target_os = "windows") {
            self.replace("\\", "%5C")
                .replace("/", "%2F")
                .replace(":", "%3A")
        } else {
            self.replace("/", "%2F")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FilesystemSafeEscaper;
    #[test]
    fn test_escape() {
        assert_eq!(
            String::from("img-98/12/12.jpg").escape(),
            String::from("img-98%2F12%2F12.jpg")
        );

        assert_eq!(
            String::from("img:23.jpg").escape(),
            String::from("img:23.jpg")
        );

        assert_eq!(
            String::from("img\\23.jpg").escape(),
            String::from("img\\23.jpg")
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_escape_windows() {
        assert_eq!(
            String::from("img-98/12/12.jpg").escape(),
            String::from("img-98%2F12%2F12.jpg")
        );

        assert_eq!(
            String::from("img:23.jpg").escape(),
            String::from("img%3A23.jpg")
        );

        assert_eq!(
            String::from("img\\23.jpg").escape(),
            String::from("img%5C23.jpg")
        );
    }
}
