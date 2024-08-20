use std::{io, path};

use actix_multipart::form::tempfile::TempFile;
use mime::{self, IMAGE, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

pub const MAX_SIZE_DEF: u32 = 0;
pub const MAX_WIDTH_DEF: u32 = 0;
pub const MAX_HEIGHT_DEF: u32 = 0;

#[derive(Debug, Clone)]
pub struct ConfigFile {
    // Directory for storing files.
    pub file_dir: String,
    // Maximum size for files.
    pub max_size: usize,
    // List of valid input mime types for files (comma delimited).
    pub valid_types: Vec<String>,
    // Files will be converted to this MIME type.
    // Valid values: jpeg,gif,png,bmp
    pub file_ext: Option<String>,
    // Maximum width for a file.
    pub max_width: u32,
    // Maximum height for a file.
    pub max_height: u32,
}

impl ConfigFile {
    pub fn new(file_dir: &str, max_size: Option<usize>, file_ext: Option<String>) -> Self {
        ConfigFile {
            file_dir: file_dir.to_string(),
            max_size: max_size.unwrap_or(MAX_SIZE_DEF as usize),
            valid_types: Self::get_image_types(),
            file_ext,
            max_width: MAX_WIDTH_DEF,
            max_height: MAX_HEIGHT_DEF,
        }
    }
    pub fn get_image_types() -> Vec<String> {
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
}

pub enum ErrUpload {
    #[rustfmt::skip]
    InvalidFileSize { actual_size: usize, max_size: usize },
    #[rustfmt::skip]
    InvalidFileType { actual_type: String, valid_types: String },
    FileNameNotDefined,
    #[rustfmt::skip]
    ErrorSavingFile {path_file: String, error: io::Error},
}

pub struct UploadFile {
    config_file: ConfigFile,
}

impl UploadFile {
    pub fn new(config: ConfigFile) -> Self {
        UploadFile { config_file: config }
    }
    #[rustfmt::skip]
    pub fn upload(&self, temp_file: TempFile, only_file_name: &str) -> Result<Option<String>, ErrUpload> {

        if temp_file.size == 0 {
            // Delete the old version of the file.
            return Ok(None);
        }
        // Check the name of the new file that it is not empty.
        if only_file_name.len() == 0 {
            return Err(ErrUpload::FileNameNotDefined);
        }

        // Check file size for maximum value.
        if self.config_file.max_size > 0 && temp_file.size > self.config_file.max_size {
            // return Err(AppError::content_large413(MSG_INVALID_FILE_SIZE) // 413
            #[rustfmt::skip]
            return Err(ErrUpload::InvalidFileSize { actual_size: temp_file.size, max_size: self.config_file.max_size });
        }

        // Checking the file for valid mime types.
        let file_content_type = temp_file.content_type.clone();
        let file_mime_type = match file_content_type { Some(v) => v.to_string(), None => "".to_string() };
        let valid_file_types: Vec<String> = self.config_file.valid_types.clone();
        if !valid_file_types.contains(&file_mime_type) {
            // return Err(AppError::unsupported_type415(MSG_INVALID_FILE_TYPE) // 415
            #[rustfmt::skip]
            return Err(ErrUpload::InvalidFileType { actual_type: file_mime_type, valid_types: valid_file_types.join(",") });
        }

        // Get a new file extension.
        let file_ext = file_mime_type.replace(&format!("{}/", IMAGE), "");
        let new_file_name_and_ext = format!("{}.{}", only_file_name, &file_ext);
        // Add 'file path' + 'file name'.'file extension'.
        let path: path::PathBuf = [&self.config_file.file_dir, &new_file_name_and_ext].iter().collect();
        // Get the full name for a new file.
        let full_path_file: String = path.to_str().unwrap().to_string();

        // Persist the temporary file at the target path.
        // If a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            // return Err(AppError::internal_err500(&message)) // 500
            return Err(ErrUpload::ErrorSavingFile { path_file: full_path_file, error: err.error });
        }
        Ok(Some(full_path_file))
    }
}
