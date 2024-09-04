use std::{io, path::PathBuf};

use mime::{self, IMAGE, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

const PRFL_AVATAR_MAX_SIZE_DEF: u32 = 0;
const PRFL_AVATAR_MAX_WIDTH_DEF: u32 = 0;
const PRFL_AVATAR_MAX_HEIGHT_DEF: u32 = 0;

// Profile Properties
#[derive(Debug, Clone)]
pub struct ConfigPrfl {
    // Directory for storing avatar files.
    pub prfl_avatar_files_dir: String,
    // Maximum size for avatar files.
    pub prfl_avatar_max_size: u32,
    // List of valid input mime types for avatar files.
    // ["image/bmp", "image/gif", "image/jpeg", "image/png"]
    pub prfl_avatar_valid_types: Vec<String>,
    // Avatar files will be converted to this MIME type.
    // Valid values: "image/bmp", "image/gif", "image/jpeg", "image/png"
    pub prfl_avatar_ext: Option<String>,
    // Maximum width of avatar image after saving.
    pub prfl_avatar_max_width: u32,
    // Maximum height of avatar image after saving.
    pub prfl_avatar_max_height: u32,
}

impl ConfigPrfl {
    pub fn init_by_env() -> Self {
        let avatar_files_dir = std::env::var("PRFL_AVATAR_FILES_DIR").expect("PRFL_AVATAR_FILES_DIR must be set");
        let path_dir: PathBuf = PathBuf::from(avatar_files_dir).iter().collect();
        let prfl_avatar_files_dir = path_dir.to_str().unwrap().to_string();

        let max_size_def = PRFL_AVATAR_MAX_SIZE_DEF.to_string();
        let avatar_max_size = std::env::var("PRFL_AVATAR_MAX_SIZE")
            .unwrap_or(max_size_def)
            .trim()
            .parse()
            .unwrap();

        let valid_types = Self::get_avatar_valid_types();
        let avatar_valid_types: Vec<String> = Self::get_avatar_valid_types_by_str(&valid_types).unwrap();

        let avatar_ext = std::env::var("PRFL_AVATAR_EXT").unwrap_or("".to_string());
        #[rustfmt::skip]
        let prfl_avatar_ext = if Self::avatar_ext_validate(&avatar_ext) { Some(avatar_ext) } else { None };

        let max_width_def = PRFL_AVATAR_MAX_WIDTH_DEF.to_string();
        #[rustfmt::skip]
        let avatar_max_width: u32 = std::env::var("PRFL_AVATAR_MAX_WIDTH")
        .unwrap_or(max_width_def).trim().parse().unwrap();

        let max_height_def = PRFL_AVATAR_MAX_HEIGHT_DEF.to_string();
        #[rustfmt::skip]
        let avatar_max_height: u32 = std::env::var("PRFL_AVATAR_MAX_HEIGHT")
            .unwrap_or(max_height_def).trim().parse().unwrap();

        ConfigPrfl {
            prfl_avatar_files_dir,
            prfl_avatar_max_size: avatar_max_size,
            prfl_avatar_valid_types: avatar_valid_types,
            prfl_avatar_ext,
            prfl_avatar_max_width: avatar_max_width,
            prfl_avatar_max_height: avatar_max_height,
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
    pub fn get_avatar_valid_types() -> String {
        std::env::var("PRFL_AVATAR_VALID_TYPES").expect("PRFL_AVATAR_VALID_TYPES must be set")
    }
    pub fn get_avatar_valid_types_by_str(avatar_valid_types: &str) -> Result<Vec<String>, io::Error> {
        let image_types: Vec<String> = Self::image_types();

        let mut errors: Vec<String> = Vec::new();
        let mut result: Vec<String> = Vec::new();
        for strm_type in avatar_valid_types.split(",").into_iter() {
            let value = strm_type.to_lowercase();
            if image_types.contains(&value) {
                result.push(value);
            } else {
                errors.push(value);
            }
        }
        if errors.len() > 0 {
            let msg = format!("Incorrect values for \"PRFL_AVATAR_VALID_TYPES\": {}", errors.join(","));
            return Err(io::Error::new(io::ErrorKind::Other, msg));
        }
        Ok(result)
    }
    fn avatar_ext_validate(value: &str) -> bool {
        let type_list: Vec<String> = Self::get_types(Self::image_types());
        value.len() > 0 && type_list.contains(&value.to_string())
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigPrfl {
    ConfigPrfl {
        prfl_avatar_files_dir: "./tmp".to_string(),
        prfl_avatar_max_size: PRFL_AVATAR_MAX_SIZE_DEF,
        prfl_avatar_valid_types: vec![
            IMAGE_JPEG.essence_str().to_string(),
            IMAGE_PNG.essence_str().to_string(),
        ],
        prfl_avatar_ext: None,
        prfl_avatar_max_width: PRFL_AVATAR_MAX_WIDTH_DEF,
        prfl_avatar_max_height: PRFL_AVATAR_MAX_HEIGHT_DEF,
    }
}
