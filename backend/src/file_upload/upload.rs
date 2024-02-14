use std::{self, ffi, path::PathBuf};

use actix_multipart::form::tempfile::TempFile;
use image::{imageops::FilterType, DynamicImage, ImageFormat};

// Upload a file with the specified name.
pub fn file_upload(temp_file: TempFile, dir_path: &str, file_name: &str) -> Result<String, String> {
    if temp_file.file_name.is_none() {
        return Err("The name for the upload file is missing.".to_string());
    }
    let path = PathBuf::from(temp_file.file_name.unwrap());
    let old_file_ext = path.extension().unwrap_or(ffi::OsStr::new("")).to_str().unwrap().to_string();

    let mut path_buf = PathBuf::from(dir_path);
    path_buf.push(file_name);
    path_buf.set_extension(old_file_ext);
    let path_file = path_buf.to_str().unwrap();
    let result = temp_file.file.persist(path_file);
    if let Err(err) = result {
        return Err(format!("{}: {}", err.to_string(), &path_file));
    }
    Ok(path_file.to_string())
}

pub fn convert(source: &str, receiver: &str) -> Result<(), String> {
    let path_source = PathBuf::from(source);
    let source_extension = path_source.extension().unwrap().to_str().unwrap().to_string();
    // Check that the image type of the source file is correct.
    let opt_source_type = ImageFormat::from_extension(source_extension);
    if opt_source_type.is_none() {
        return Err(format!("Invalid source file image type \"{}\".", source));
    }
    let source_type = opt_source_type.unwrap();

    let path_receiver = PathBuf::from(receiver);
    let receiver_extension = path_receiver.extension().unwrap().to_str().unwrap().to_string();
    // Check that the image type of the receiver file is correct.
    let opt_receiver_type = ImageFormat::from_extension(receiver_extension);
    if opt_receiver_type.is_none() {
        return Err(format!("Invalid receiver file image type \"{}\".", receiver));
    }
    let receiver_type = opt_receiver_type.unwrap();

    if source_type == receiver_type {
        return Err("The source and destination have the same image type.".to_string());
    }
    // Load the source image into memory.
    let image_data: DynamicImage = image::open(source).unwrap();

    // Delete the source image file.
    // let _ = fs::remove_file(source).await.unwrap();

    // Save the image from memory to the receiver.
    image_data.resize_exact(200, 200, FilterType::Gaussian).save(receiver).unwrap();

    Ok(())
}
