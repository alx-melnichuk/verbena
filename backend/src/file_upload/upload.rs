use actix_multipart::form::tempfile::TempFile;

use crate::streams::config_avatar_files::ConfigAvatarFiles;

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

pub fn file_validate_size(temp_file: &TempFile, max_size: usize) -> Result<(), usize> {
    if temp_file.size > max_size {
        return Err(temp_file.size);
    }
    Ok(())
}

pub fn file_upload(temp_file: TempFile, config_af: ConfigAvatarFiles, file_name: String) -> Result<(), String> {
    let avatar_dir = config_af.avatar_dir.to_string();
    let path_file = format!("{}{}", avatar_dir, file_name);
    temp_file.file.persist(path_file);

    Ok(())
}
