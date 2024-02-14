use std::{io, path::PathBuf};

use mime::{self, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

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

        let slp_valid_types: Vec<String> = Self::init_slp_valid_types_by_env().unwrap();

        ConfigSLP {
            slp_dir,
            slp_max_size: slp_max_size.parse::<usize>().unwrap(),
            slp_valid_types,
        }
    }

    pub fn init_slp_valid_types_by_env() -> Result<Vec<String>, io::Error> {
        let acceptable_types: Vec<&str> = vec![
            IMAGE_BMP.essence_str(),
            IMAGE_GIF.essence_str(),
            IMAGE_JPEG.essence_str(),
            IMAGE_PNG.essence_str(),
        ];

        let slp_valid_types_str = std::env::var("SLP_FILES_VALID_TYPES").expect("SLP_FILES_VALID_TYPES must be set");
        let mut errors: Vec<String> = Vec::new();
        let mut result: Vec<String> = Vec::new();
        for slp_type in slp_valid_types_str.split(",").into_iter() {
            let value = slp_type.to_lowercase();
            if acceptable_types.contains(&value.as_str()) {
                result.push(value);
            } else {
                errors.push(value);
            }
        }
        if errors.len() > 0 {
            let msg = format!("Incorrect values for \"SLP_FILES_VALID_TYPES\": {}", errors.join(","));
            return Err(io::Error::new(io::ErrorKind::Other, msg));
        }
        Ok(result)
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
