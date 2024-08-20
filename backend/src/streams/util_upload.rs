use std::convert::From;

use crate::loading::upload_file::ConfigFile;
use crate::streams::config_strm::ConfigStrm;

///
///  let config_file = ConfigFile::from(config_strm);
///

impl From<ConfigStrm> for ConfigFile {
    fn from(config_strm: ConfigStrm) -> Self {
        ConfigFile {
            // Directory for storing files.
            file_dir: config_strm.strm_logo_files_dir.clone(),
            // Maximum size for files.
            max_size: config_strm.strm_logo_max_size,
            // List of valid input mime types for files (comma delimited).
            valid_types: config_strm.strm_logo_valid_types.clone(),
            // Files will be converted to this MIME type.
            // Valid values: jpeg,gif,png,bmp
            file_ext: config_strm.strm_logo_ext.clone(),
            // Maximum width for a file.
            max_width: config_strm.strm_logo_max_width,
            // Maximum height for a file.
            max_height: config_strm.strm_logo_max_height,
        }
    }
}
