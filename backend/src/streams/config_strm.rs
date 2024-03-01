use std::{io, path::PathBuf};

use mime::{self, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

const STRM_SHOW_LEAD_TIME_DEF: bool = false;

// Stream Logo Properties
#[derive(Debug, Clone)]
pub struct ConfigStrm {
    // A flag to display the execution time of methods.
    pub strm_show_lead_time: bool,
    // Directory for storing logo files.
    pub strm_logo_files_dir: String, // slp_dir
    // Maximum size for logo files.
    pub strm_logo_max_size: usize,
    // List of valid input mime types for logo files (comma delimited).
    pub strm_logo_valid_types: Vec<String>,
}

impl ConfigStrm {
    pub fn init_by_env() -> Self {
        let def = STRM_SHOW_LEAD_TIME_DEF.to_string();
        let strm_show_lead_time: bool = std::env::var("STRM_SHOW_LEAD_TIME").unwrap_or(def).trim().parse().unwrap();

        let logo_files_dir = std::env::var("STRM_LOGO_FILES_DIR").expect("STRM_LOGO_FILES_DIR must be set");
        let path_dir: PathBuf = PathBuf::from(logo_files_dir).iter().collect();
        let strm_logo_files_dir = path_dir.to_str().unwrap().to_string();

        let logo_max_size = std::env::var("STRM_LOGO_MAX_SIZE").expect("STRM_LOGO_MAX_SIZE must be set");

        let strm_logo_valid_types: Vec<String> = Self::init_strm_valid_types_by_env().unwrap();

        ConfigStrm {
            strm_show_lead_time,
            strm_logo_files_dir,
            strm_logo_max_size: logo_max_size.parse::<usize>().unwrap(),
            strm_logo_valid_types,
        }
    }

    pub fn init_strm_valid_types_by_env() -> Result<Vec<String>, io::Error> {
        let acceptable_types: Vec<&str> = vec![
            IMAGE_BMP.essence_str(),
            IMAGE_GIF.essence_str(),
            IMAGE_JPEG.essence_str(),
            IMAGE_PNG.essence_str(),
        ];

        let strm_valid_types_str = std::env::var("STRM_LOGO_VALID_TYPES").expect("STRM_LOGO_VALID_TYPES must be set");
        let mut errors: Vec<String> = Vec::new();
        let mut result: Vec<String> = Vec::new();
        for strm_type in strm_valid_types_str.split(",").into_iter() {
            let value = strm_type.to_lowercase();
            if acceptable_types.contains(&value.as_str()) {
                result.push(value);
            } else {
                errors.push(value);
            }
        }
        if errors.len() > 0 {
            let msg = format!("Incorrect values for \"STRM_LOGO_VALID_TYPES\": {}", errors.join(","));
            return Err(io::Error::new(io::ErrorKind::Other, msg));
        }
        Ok(result)
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigStrm {
    ConfigStrm {
        strm_show_lead_time: false,
        strm_logo_files_dir: "./tmp".to_string(),
        strm_logo_max_size: 160,
        strm_logo_valid_types: vec!["image/jpeg".to_string(), "image/png".to_string()],
    }
}
