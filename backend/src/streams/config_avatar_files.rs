use actix_multipart::form::tempfile::TempFileConfig;

#[derive(Debug, Clone)]
pub struct ConfigAvatarFiles {
    // Directory for storing temporary files.
    pub avatar_dir_tmp: String,
    // Directory for storing avatar files.
    pub avatar_dir: String,
    // Maximum size for avatar files.
    pub avatar_max_size: usize,
    // List of valid input mime types for avatar files (comma delimited).
    pub avatar_valid_types: Vec<String>,
    // To reduce the size, the avatar file is saved with the specified mime type.
    pub avatar_type_to_save: String,
}

impl ConfigAvatarFiles {
    pub fn init_by_env() -> Self {
        let avatar_dir_tmp = std::env::var("AVATAR_FILES_DIR_TMP").expect("AVATAR_FILES_DIR_TMP must be set");
        let avatar_dir = std::env::var("AVATAR_FILES_DIR").expect("AVATAR_FILES_DIR must be set");
        let avatar_max_size = std::env::var("AVATAR_FILES_MAX_SIZE").expect("AVATAR_FILES_MAX_SIZE must be set");
        let avatar_valid_types_str =
            std::env::var("AVATAR_FILES_VALID_TYPES").expect("AVATAR_FILES_VALID_TYPES must be set");
        let avatar_valid_types = avatar_valid_types_str.split(",").into_iter().map(|v| v.to_string()).collect();
        let avatar_type_to_save =
            std::env::var("AVATAR_FILES_TYPE_TO_SAVE").expect("AVATAR_FILES_TYPE_TO_SAVE must be set");

        ConfigAvatarFiles {
            avatar_dir_tmp,
            avatar_dir,
            avatar_max_size: avatar_max_size.parse::<usize>().unwrap(),
            avatar_valid_types,
            avatar_type_to_save,
        }
    }

    pub fn get_temp_file_config(&self) -> TempFileConfig {
        TempFileConfig::default().clone().directory(self.avatar_dir_tmp.to_string())
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigAvatarFiles {
    ConfigAvatarFiles {
        avatar_dir: "./tmp".to_string(),
        avatar_max_size: (1 * 1024 * 1024),
        avatar_valid_types: vec!["jpeg".to_string()],
        avatar_type_to_save: "png".to_string(),
    }
}
