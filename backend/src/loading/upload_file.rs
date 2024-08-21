use std::path;

use actix_multipart::form::tempfile::TempFile;
use chrono::Utc;
use mime::{self, IMAGE, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

use crate::cdis::coding;

pub const MAX_SIZE_DEF: u32 = 0;

#[derive(Debug, Clone)]
pub enum ErrUpload {
    // ErrUpload into AppError
    #[rustfmt::skip]
    InvalidFileSize { actual_size: usize, max_size: usize },
    #[rustfmt::skip]
    InvalidFileType { actual_type: String, valid_types: String },
    #[rustfmt::skip]
    ErrorSavingFile {path_file: String, err: String},
}

#[derive(Debug, Clone)]
pub struct UploadFile {
    // Maximum size for files.
    pub max_size: usize,
    // List of valid input mime types for files.
    pub valid_mime_types: Vec<String>,
    // Directory for storing files.
    pub file_dir: String,
}

impl UploadFile {
    pub fn new(opt_max_size: Option<usize>, valid_mime_types: Vec<String>, file_dir: String) -> Self {
        UploadFile {
            max_size: opt_max_size.unwrap_or(MAX_SIZE_DEF as usize),
            valid_mime_types,
            file_dir,
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
    #[rustfmt::skip]
    pub fn upload(&self, temp_file: TempFile, file_stem: &str) -> Result<Option<String>, ErrUpload> {
        if temp_file.size == 0 {
            // Delete the old version of the file.
            return Ok(None);
        }
        let tmp_file_stem = coding::encode(Utc::now(), 1);

        // Get the name stem for the new file.
        let curr_file_stem = if file_stem.len() > 0 {
            file_stem.to_string()
        } else {
            temp_file.file_name.unwrap_or(tmp_file_stem)
        };

        // Check the size for the new file to the maximum value.
        if self.max_size > 0 && temp_file.size > self.max_size {
            #[rustfmt::skip]
            return Err(ErrUpload::InvalidFileSize { actual_size: temp_file.size, max_size: self.max_size });
        }
        // Get the MIME type of the new file.
        #[rustfmt::skip]
        let file_mime_type = match temp_file.content_type { Some(v) => v.to_string(), None => "".to_string() };

        // Checks the new file type against the list of valid MIME types.
        if self.valid_mime_types.len() > 0 && file_mime_type.len() > 0 && !self.valid_mime_types.contains(&file_mime_type) {
            #[rustfmt::skip]
            return Err(ErrUpload::InvalidFileType { actual_type: file_mime_type, valid_types: self.valid_mime_types.join(",") });
        }

        // Get the extension for the new file.
        let file_ext = file_mime_type.replace(&format!("{}/", IMAGE), "");
        // Get the name and extension for the new file.
        let new_file_name_and_ext = format!("{}.{}", curr_file_stem, &file_ext);

        // Get the full path for the new file. ('file path' + 'file name'.'file extension')
        let path: path::PathBuf = [&self.file_dir, &new_file_name_and_ext].iter().collect();
        let full_path_file: String = path.to_str().unwrap().to_string();

        // Persist the new file at the target path.
        // If a file exists at the target path, persist will atomically replace it.
        let res_upload = temp_file.file.persist(&full_path_file);
        if let Err(err) = res_upload {
            return Err(ErrUpload::ErrorSavingFile { path_file: full_path_file, err: err.error.to_string() });
        }

        Ok(Some(full_path_file))
    }
}
