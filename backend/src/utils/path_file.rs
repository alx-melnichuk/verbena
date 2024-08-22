use std::path;

// Get the file path from the full file path.
pub fn get_file_path(full_file_name: &str) -> Option<String> {
    path::PathBuf::from(full_file_name)
        .parent()
        .map(|s| s.to_str().unwrap().to_string())
}

// Get the file name (with extension) from the full path to the file.
pub fn get_file_name(full_file_name: &str) -> Option<String> {
    path::PathBuf::from(full_file_name)
        .file_name()
        .map(|s| s.to_str().unwrap().to_string())
}

// Get the filename (without extension) from the full path to the file.
pub fn get_file_stem(full_file_name: &str) -> Option<String> {
    path::PathBuf::from(full_file_name)
        .file_stem()
        .map(|s| s.to_str().unwrap().to_string())
}

// Get file extension from full file path.
pub fn get_file_ext(full_file_name: &str) -> Option<String> {
    path::PathBuf::from(full_file_name)
        .extension()
        .map(|s| s.to_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: &str = "./temp1";
    const NAME: &str = "test";
    const EXT: &str = "txt";

    // ** get_file_path **
    #[test]
    fn test_get_file_path_is_not_empty_path() {
        let full_file_name = format!("{}/{}.{}", PATH, NAME, EXT);
        let result = get_file_path(full_file_name.as_str());

        assert_eq!(result, Some(PATH.to_string()));
    }
    #[test]
    fn test_get_file_path_is_not_empty_path2() {
        let path1 = format!("{}/{}", PATH, NAME);
        let full_file_name = format!("{}/{}", path1, EXT);
        let result = get_file_path(full_file_name.as_str());

        assert_eq!(result, Some(path1.to_string()));
    }
    #[test]
    fn test_get_file_path_is_empty_path() {
        let full_file_name = format!("/temp1");
        let result = get_file_path(full_file_name.as_str());

        assert_eq!(result, Some("/".to_string()));
    }
    #[test]
    fn test_get_file_path_is_empty_path2() {
        let full_file_name = format!("/");
        let result = get_file_path(full_file_name.as_str());

        assert_eq!(result, None);
    }
    #[test]
    fn test_get_file_path_is_empty_path3() {
        let full_file_name = format!("temp1");
        let result = get_file_path(full_file_name.as_str());

        assert_eq!(result, Some("".to_string()));
    }

    // ** get_file_name **
    #[test]
    fn test_get_file_name_is_not_empty_name() {
        let file_name = format!("{}.{}", NAME, EXT);
        let full_file_name = format!("{}/{}", PATH, &file_name);
        let result = get_file_name(full_file_name.as_str());

        assert_eq!(result, Some(file_name));
    }
    #[test]
    fn test_get_file_name_is_not_empty_name2() {
        let file_name = format!("{}/", NAME);
        let full_file_name = format!("{}/{}", PATH, &file_name);
        let result = get_file_name(full_file_name.as_str());

        assert_eq!(result, Some(NAME.to_string()));
    }
    #[test]
    fn test_get_file_name_is_empty_name() {
        let full_file_name = format!("{}.{}/..", NAME, EXT);
        let result = get_file_name(full_file_name.as_str());

        assert_eq!(result, None);
    }
    #[test]
    fn test_get_file_name_is_empty_name2() {
        let full_file_name = format!("/");
        let result = get_file_name(full_file_name.as_str());

        assert_eq!(result, None);
    }

    // ** get_file_stem **
    #[test]
    fn test_get_file_stem_is_not_empty_name() {
        let file_name = format!("{}.{}", NAME, EXT);
        let full_file_name = format!("{}/{}", PATH, &file_name);
        let result = get_file_stem(full_file_name.as_str());

        assert_eq!(result, Some(NAME.to_string()));
    }
    #[test]
    fn test_get_file_stem_is_not_empty_name2() {
        let file_name = format!("{}/", NAME);
        let full_file_name = format!("{}/{}", PATH, &file_name);
        let result = get_file_stem(full_file_name.as_str());

        assert_eq!(result, Some(NAME.to_string()));
    }
    #[test]
    fn test_get_file_stem_is_empty_ext() {
        let full_file_name = format!("{}.{}/..", NAME, EXT);
        let result = get_file_stem(full_file_name.as_str());

        assert_eq!(result, None);
    }
    #[test]
    fn test_get_file_stem_is_empty_name() {
        let full_file_name = format!("/");
        let result = get_file_stem(full_file_name.as_str());

        assert_eq!(result, None);
    }

    // ** get_file_ext **
    #[test]
    fn test_get_file_ext_is_not_empty_ext() {
        let full_file_name = format!("{}/{}.{}", PATH, NAME, EXT);
        let result = get_file_ext(full_file_name.as_str());

        assert_eq!(result, Some(EXT.to_string()));
    }
    #[test]
    fn test_get_file_ext_is_empty_ext() {
        let full_file_name = format!("{}/{}.", PATH, NAME);
        let result = get_file_ext(full_file_name.as_str());

        assert_eq!(result, Some("".to_string()));
    }
}
