use std::{env, io, path::PathBuf};

use mime::{self, IMAGE, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};
use vrb_common::consts;

pub const LOGO_MAX_SIZE: &str = "0";
pub const LOGO_VALID_TYPES: &str = "image/jpeg,image/gif,image/png,image/bmp";
pub const LOGO_MAX_WIDTH: &str = "0";
pub const LOGO_MAX_HEIGHT: &str = "0";

// Stream Logo Properties
#[derive(Debug, Clone)]
pub struct ConfigStrm {
    // Directory for storing logo files.
    pub strm_logo_files_dir: String,
    // Maximum size for logo files.
    pub strm_logo_max_size: u32,
    // List of valid input mime types for logo files.
    // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
    pub strm_logo_valid_types: Vec<String>,
    // Logo files will be converted to this MIME type.
    // Valid values: jpeg,gif,png,bmp
    pub strm_logo_ext: Option<String>,
    // Maximum width for a logo file.
    pub strm_logo_max_width: u32,
    // Maximum height for a logo file.
    pub strm_logo_max_height: u32,
}

impl ConfigStrm {
    pub fn init_by_env() -> Self {
        let logo_files_dir = env::var(consts::STRM_LOGO_FILES_DIR).unwrap_or(consts::LOGO_FILES_DIR.to_string());
        let path_dir: PathBuf = PathBuf::from(logo_files_dir).iter().collect();
        let strm_logo_files_dir = path_dir.to_str().unwrap().to_string();

        let max_size = LOGO_MAX_SIZE.to_string();
        let logo_max_size = env::var(consts::STRM_LOGO_MAX_SIZE).unwrap_or(max_size).trim().parse().unwrap();

        #[rustfmt::skip]
        let valid_types = env::var(consts::STRM_LOGO_VALID_TYPES).unwrap_or(LOGO_VALID_TYPES.to_string()).to_lowercase();
        let logo_valid_types: Vec<String> = Self::get_logo_valid_types_by_str(&valid_types).unwrap();

        let logo_ext = env::var(consts::STRM_LOGO_EXT).unwrap_or("".to_string());
        #[rustfmt::skip]
        let strm_logo_ext = if Self::logo_ext_validate(&logo_ext) { Some(logo_ext) } else { None };

        let max_width = LOGO_MAX_WIDTH.to_string();
        #[rustfmt::skip]
        let logo_max_width: u32 = env::var(consts::STRM_LOGO_MAX_WIDTH).unwrap_or(max_width).trim().parse().unwrap();

        let max_height = LOGO_MAX_HEIGHT.to_string();
        #[rustfmt::skip]
        let logo_max_height: u32 = env::var(consts::STRM_LOGO_MAX_HEIGHT).unwrap_or(max_height).trim().parse().unwrap();

        ConfigStrm {
            strm_logo_files_dir,
            strm_logo_max_size: logo_max_size,
            strm_logo_valid_types: logo_valid_types,
            strm_logo_ext,
            strm_logo_max_width: logo_max_width,
            strm_logo_max_height: logo_max_height,
        }
    }

    pub fn image_types() -> Vec<String> {
        vec![
            IMAGE_BMP.essence_str().to_string(),
            IMAGE_GIF.essence_str().to_string(),
            IMAGE_JPEG.essence_str().to_string(),
            IMAGE_PNG.essence_str().to_string(),
        ]
    }
    pub fn get_types(image_types: Vec<String>) -> Vec<String> {
        let text = format!("{}/", IMAGE);
        let types: Vec<String> = image_types.iter().map(|v| v.replace(&text, "")).collect();
        types
    }
    pub fn get_logo_valid_types_by_str(logo_valid_types: &str) -> Result<Vec<String>, io::Error> {
        let image_types: Vec<String> = Self::image_types();

        let mut errors: Vec<String> = Vec::new();
        let mut result: Vec<String> = Vec::new();
        for strm_type in logo_valid_types.split(",").into_iter() {
            let value = strm_type.to_lowercase();
            if image_types.contains(&value) {
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
    fn logo_ext_validate(value: &str) -> bool {
        let type_list: Vec<String> = Self::get_types(Self::image_types());
        value.len() > 0 && type_list.contains(&value.to_string())
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigStrm {
    ConfigStrm {
        strm_logo_files_dir: "./tmp".to_string(),
        strm_logo_max_size: LOGO_MAX_SIZE.parse().unwrap(),
        strm_logo_valid_types: vec![IMAGE_JPEG.essence_str().to_string(), IMAGE_PNG.essence_str().to_string()],
        strm_logo_ext: None,
        strm_logo_max_width: LOGO_MAX_WIDTH.parse().unwrap(),
        strm_logo_max_height: LOGO_MAX_HEIGHT.parse().unwrap(),
    }
}
