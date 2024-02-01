use std::path::PathBuf;

// Stream Logo Properties
#[derive(Debug, Clone)]
pub struct ConfigSLP {
    // Directory for storing logo files.
    pub slp_dir: String,
    // Maximum size for logo files.
    pub slp_max_size: usize,
    // List of valid input mime types for logo files (comma delimited).
    pub slp_valid_types: Vec<String>,
    // To reduce the size, the logo file is saved with the specified mime type.
    pub slp_type_to_save: String,
}

impl ConfigSLP {
    pub fn init_by_env() -> Self {
        let slp_dir_str = std::env::var("SLP_FILES_DIR").expect("SLP_FILES_DIR must be set");
        let path_dir: PathBuf = PathBuf::from(slp_dir_str).iter().collect();
        let slp_dir = path_dir.to_str().unwrap().to_string();

        let slp_max_size = std::env::var("SLP_FILES_MAX_SIZE").expect("SLP_FILES_MAX_SIZE must be set");
        let slp_valid_types_str = std::env::var("SLP_FILES_VALID_TYPES").expect("SLP_FILES_VALID_TYPES must be set");
        let slp_valid_types = slp_valid_types_str.split(",").into_iter().map(|v| v.to_string()).collect();
        let slp_type_to_save = std::env::var("SLP_FILES_TYPE_TO_SAVE").expect("SLP_FILES_TYPE_TO_SAVE must be set");

        ConfigSLP {
            slp_dir,
            slp_max_size: slp_max_size.parse::<usize>().unwrap(),
            slp_valid_types,
            slp_type_to_save,
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigSLP {
    ConfigSLP {
        slp_dir: "./tmp".to_string(),
        slp_max_size: (1 * 1024 * 1024),
        slp_valid_types: vec!["jpeg".to_string()],
        slp_type_to_save: "png".to_string(),
    }
}
