use std::{self, ffi::OsStr, path::PathBuf};

use image::{DynamicImage, GenericImageView, ImageFormat};

pub const MSG_INVALID_SOURCE_IMAGE_TYPE: &str = "Invalid source file image type ";
pub const MSG_INVALID_RECEIVER_IMAGE_TYPE: &str = "Invalid receiver file image type ";

/** Convert the file to another mime type. */
pub fn convert_file(source: &str, extension: &str, max_width: u32, max_height: u32) -> Result<String, String> {
    let mut path = PathBuf::from(source);
    let source_ext = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap().to_string();

    // Check the input file image type is correct.
    let opt_source_type = ImageFormat::from_extension(&source_ext);
    if opt_source_type.is_none() {
        return Err(format!("{}\"{}\".", MSG_INVALID_SOURCE_IMAGE_TYPE, &source_ext));
    }
    // Check the output file image type is correct.
    let opt_receiver_type = ImageFormat::from_extension(extension);
    if opt_receiver_type.is_none() {
        return Err(format!("{}\"{}\".", MSG_INVALID_RECEIVER_IMAGE_TYPE, extension));
    }
    // If the image type does not change and the maximum dimensions are not specified, then exit.
    if source_ext.eq(extension) && max_width == 0 && max_height == 0 {
        return Ok(source.to_string());
    }
    path.set_extension(extension);
    let receiver = path.to_str().unwrap();

    // Load the source image into memory.
    let mut image_source: DynamicImage = image::open(source).map_err(|err| err.to_string())?;
    // Get the width and height of this image.
    let (curr_width, curr_height) = image_source.dimensions();
    #[rustfmt::skip]
    let new_width: u32 = if max_width > 0 && max_width < curr_width { max_width } else { 0 };
    #[rustfmt::skip]
    let new_height: u32 = if max_height > 0 && max_height < curr_height { max_height } else { 0 };

    if new_width > 0 || new_height > 0 {
        let nwidth: u32 = if new_width > 0 { new_width } else { curr_width };
        let nheight: u32 = if new_height > 0 { new_height } else { curr_height };

        // Scale this image down to fit within a specific size. Returns a new image.
        // The image's aspect ratio is preserved.
        // The image is scaled to the maximum possible size that fits within the bounds specified by nwidth and nheight.
        image_source = image_source.thumbnail(nwidth, nheight);
    }

    // Save the image from memory to the receiver.
    image_source.save(receiver).map_err(|err| err.to_string())?;

    Ok(receiver.to_string())
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use std::{fs, io::Write, path};

    use crate::streams::config_strm;

    use super::*;

    fn file_path(file_name: &str, file_ext: &str) -> String {
        let logo_files_dir = config_strm::get_test_config().strm_logo_files_dir;
        let path: path::PathBuf = [logo_files_dir, format!("{}.{}", file_name, file_ext)].iter().collect();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_convert_file_bad_source_ext() {
        let source_ext: &str = "jpegQW";

        let result = convert_file(&file_path("demo", source_ext), "", 0, 0);
        assert!(result.is_err());
        #[rustfmt::skip]
        assert_eq!(result.unwrap_err(), format!("{}\"{}\".", MSG_INVALID_SOURCE_IMAGE_TYPE, &source_ext));
    }
    #[test]
    fn test_convert_file_bad_receiver_ext() {
        let receiver_ext = "jpegQW";

        let result = convert_file(&file_path("demo", "jpeg"), receiver_ext, 0, 0);
        assert!(result.is_err());
        #[rustfmt::skip]
        assert_eq!(result.unwrap_err(), format!("{}\"{}\".", MSG_INVALID_RECEIVER_IMAGE_TYPE, &receiver_ext));
    }
    #[test]
    fn test_convert_file_source_jpeg_receiver_jpeg_maxwd_0_maxhg_0() {
        let source = file_path("demo", "jpeg");

        let result = convert_file(&source, "jpeg", 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or("".to_string()), source);
    }
    #[test]
    fn test_convert_file_source_no_exist() {
        let source = file_path("demo01", "png");
        if path::Path::new(&source).exists() {
            let _ = fs::remove_file(&source);
        }

        let result = convert_file(&source, "jpeg", 0, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("No such file or directory"));
    }

    fn create_file_png(path_file: &str) -> Result<String, String> {
        let header: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        #[rustfmt::skip]
        let buf: Vec<u8> = vec![                             0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x17, 0x00, 0x00, 0x00, 0x13,  0x08, 0x06, 0x00, 0x00, 0x00, 0x7B, 0xBB, 0x96,
            0xB6, 0x00, 0x00, 0x00, 0x04, 0x73, 0x42, 0x49,  0x54, 0x08, 0x08, 0x08, 0x08, 0x7C, 0x08, 0x64,
            0x88, 0x00, 0x00, 0x00, 0x6C, 0x49, 0x44, 0x41,  0x54, 0x38, 0x8D, 0xED, 0x94, 0x4B, 0x0A, 0x80,
            0x30, 0x0C, 0x05, 0x5F, 0xC5, 0x83, 0x28, 0x78,  0x3D, 0xDB, 0xDE, 0xA4, 0x1F, 0x0F, 0x3C, 0xAE,
            0x15, 0x6B, 0xAB, 0xD0, 0x5D, 0x07, 0x02, 0x81,  0x90, 0x49, 0xC8, 0x22, 0x06, 0x40, 0x9D, 0x98,
            0x7A, 0x89, 0x87, 0x7C, 0xC8, 0xBF, 0x31, 0x97,  0x0A, 0x31, 0x66, 0xA5, 0x7C, 0xBC, 0x36, 0x3B,
            0xBB, 0xCB, 0x7B, 0x5B, 0xAC, 0x17, 0x37, 0xF7,  0xDE, 0xCA, 0xD9, 0xFD, 0xB7, 0x58, 0x92, 0x44,
            0x85, 0x10, 0x12, 0xCB, 0xBA, 0x5D, 0x22, 0x84,  0x54, 0x6B, 0x03, 0xA0, 0x2A, 0xBF, 0x0F, 0x68,
            0x15, 0x03, 0x14, 0x6F, 0x7E, 0x3F, 0xD1, 0x53,  0x5E, 0xC3, 0xC0, 0x78, 0x5C, 0x43, 0xDE, 0xC8,
            0x09, 0xFC, 0x22, 0xB8, 0x69, 0x88, 0xAE, 0x67,  0xA8 
        ];
        let footer: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];

        let mut file = fs::File::create(path_file).map_err(|e| e.to_string())?;
        file.write_all(header.as_ref()).map_err(|e| e.to_string())?;
        file.write_all(buf.as_ref()).map_err(|e| e.to_string())?;
        file.write_all(footer.as_ref()).map_err(|e| e.to_string())?;

        Ok(path_file.to_string())
    }
    fn dimensions(file_path: &str) -> (u32, u32) {
        // Load the source image into memory.
        let image_source: DynamicImage = image::open(file_path).unwrap();
        // Get the width and height of this image.
        image_source.dimensions()
    }
    #[test]
    fn test_convert_file_source_png_receiver_jpeg_maxwd_0_maxhg_0() {
        let source = file_path("demo02", "png");
        create_file_png(&source).unwrap();
        let (source_wd, source_hg) = dimensions(&source);

        let result = convert_file(&source, "jpeg", 0, 0);

        let _ = fs::remove_file(&source);

        let receiver = file_path("demo02", "jpeg");
        let (receiver_wd, receiver_hg) = dimensions(&receiver);
        let _ = fs::remove_file(&receiver);

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or("".to_string()), receiver);
        assert_eq!(source_wd, receiver_wd);
        assert_eq!(source_hg, receiver_hg);
    }
    #[test]
    fn test_convert_file_source_png_receiver_jpeg_maxwd_10_maxhg_10() {
        let source = file_path("demo03", "png");
        create_file_png(&source).unwrap();
        let (source_wd, source_hg) = dimensions(&source);

        let result = convert_file(&source, "jpeg", 10, 10);

        let _ = fs::remove_file(&source);

        let receiver = file_path("demo03", "jpeg");
        let (receiver_wd, receiver_hg) = dimensions(&receiver);
        let _ = fs::remove_file(&receiver);

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or("".to_string()), receiver);
        assert!(source_wd > receiver_wd);
        assert!(source_hg > receiver_hg);
        assert!(10 >= receiver_wd);
        assert!(10 >= receiver_hg);
    }
    #[test]
    fn test_convert_file_source_png_receiver_png_maxwd_10_maxhg_10() {
        let source = file_path("demo04", "png");
        create_file_png(&source).unwrap();
        let (source_wd, source_hg) = dimensions(&source);

        let result = convert_file(&source, "png", 10, 10);

        let receiver = source.clone();
        let (receiver_wd, receiver_hg) = dimensions(&receiver);

        let _ = fs::remove_file(&source);

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or("".to_string()), receiver);
        assert!(source_wd > receiver_wd);
        assert!(source_hg > receiver_hg);
        assert!(10 >= receiver_wd);
        assert!(10 >= receiver_hg);
    }
    #[test]
    fn test_convert_file_source_png_receiver_png_maxwd_30_maxhg_30() {
        let source = file_path("demo05", "png");
        create_file_png(&source).unwrap();
        let (source_wd, source_hg) = dimensions(&source);

        let result = convert_file(&source, "png", 30, 30);

        let receiver = source.clone();
        let (receiver_wd, receiver_hg) = dimensions(&receiver);

        let _ = fs::remove_file(&source);

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or("".to_string()), receiver);
        assert_eq!(source_wd, receiver_wd);
        assert_eq!(source_hg, receiver_hg);
    }
}
