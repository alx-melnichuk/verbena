use std::{self, path::PathBuf};

use actix_multipart::form::tempfile::TempFile;

use crate::streams::config_slp::ConfigSLP;

// Checking the file for valid mime types.
pub fn file_validate_types(temp_file: &TempFile, valid_file_types: Vec<String>) -> Result<(), String> {
    if temp_file.content_type.is_none() {
        return Err("".to_string());
    }
    let file_type = temp_file.content_type.clone().unwrap().to_string().to_lowercase();
    if !valid_file_types.contains(&file_type) {
        return Err(file_type);
    }
    Ok(())
}
// Checking the file for a valid maximum size.
pub fn file_validate_size(temp_file: &TempFile, max_size: usize) -> Result<(), usize> {
    if max_size > 0 && temp_file.size > max_size {
        return Err(temp_file.size);
    }
    Ok(())
}
// Upload a file with the specified name.
pub fn file_upload(temp_file: TempFile, config_slp: ConfigSLP, file_name: String) -> Result<String, String> {
    if temp_file.file_name.is_none() {
        return Err("The name for the upload file is missing.".to_string());
    }
    let path = PathBuf::from(temp_file.file_name.unwrap());
    let epmty = std::ffi::OsStr::new("");
    let old_file_ext = path.extension().unwrap_or(epmty).to_str().unwrap().to_string();

    let mut path_buf = PathBuf::from(&config_slp.slp_dir);
    path_buf.push(file_name);
    path_buf.set_extension(old_file_ext);
    let path_file = path_buf.to_str().unwrap();
    let result = temp_file.file.persist(path_file);
    if let Err(err) = result {
        return Err(format!("{}: {}", err.to_string(), &path_file));
    }
    Ok(path_file.to_string())
}
