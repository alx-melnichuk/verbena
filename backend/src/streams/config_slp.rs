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
}

impl ConfigSLP {
    pub fn init_by_env() -> Self {
        let slp_dir_str = std::env::var("SLP_FILES_DIR").expect("SLP_FILES_DIR must be set");
        let path_dir: PathBuf = PathBuf::from(slp_dir_str).iter().collect();
        let slp_dir = path_dir.to_str().unwrap().to_string();

        let slp_max_size = std::env::var("SLP_FILES_MAX_SIZE").expect("SLP_FILES_MAX_SIZE must be set");
        let slp_valid_types_str = std::env::var("SLP_FILES_VALID_TYPES").expect("SLP_FILES_VALID_TYPES must be set");
        let slp_valid_types = slp_valid_types_str.split(",").into_iter().map(|v| v.to_string()).collect();

        ConfigSLP {
            slp_dir,
            slp_max_size: slp_max_size.parse::<usize>().unwrap(),
            slp_valid_types,
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigSLP {
    ConfigSLP {
        slp_dir: "./tmp".to_string(),
        slp_max_size: 160,
        slp_valid_types: vec!["image/jpeg".to_string(), "image/png".to_string()],
    }
}
