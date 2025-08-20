use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

/// Get the full path of a directory without the final separator.
pub fn path_directory(path_dir: impl AsRef<Path>) -> String {
    let path_buf: PathBuf = path_dir.as_ref().to_owned().iter().collect();
    path_buf.display().to_string()
}

/// Get the full path of a directory with the final separator.
pub fn path_complete(path_dir: impl AsRef<Path>) -> String {
    let mut path_buf = path_dir.as_ref().to_owned();
    if !path_buf.ends_with(MAIN_SEPARATOR_STR) {
        path_buf.extend([""]);
    }
    path_buf.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::{path_directory, path_complete};


    #[actix_web::test]
    async fn test_path_directory() {
        let path1 = "path1";
        assert_eq!(path_directory(path1), path1);

        let path2 = "path2";
        assert_eq!(path_directory(format!("{}/", path2)), path2);

        let path3 = "path3/demo";
        assert_eq!(path_directory(path3), path3);

        let path4 = "path4/demo";
        assert_eq!(path_directory(format!("{}/", path4)), path4);
    }

    #[actix_web::test]
    async fn test_path_complete() {
        let path1 = "path1";
        assert_eq!(path_complete(path1), format!("{}/", path1));

        let path2 = "path2/";
        assert_eq!(path_complete(path2), path2);

        let path3 = "path3/demo";
        assert_eq!(path_complete(path3), format!("{}/", path3));

        let path4 = "path4/demo/";
        assert_eq!(path_complete(path4), path4);
    }
}