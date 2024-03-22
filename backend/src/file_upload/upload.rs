use std::{self, ffi::OsStr, path::PathBuf};

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};

pub fn convert_file(source: &str, receiver: &str, _min_width: u32, _min_height: u32) -> Result<String, String> {
    let path_source = PathBuf::from(source);
    let source_extension = path_source.extension().unwrap_or(OsStr::new("")).to_str().unwrap().to_string();

    // Check that the image type of the source file is correct.
    let opt_source_type = ImageFormat::from_extension(source_extension);
    if opt_source_type.is_none() {
        return Err(format!("Invalid source file image type \"{}\".", source));
    }

    let path_receiver = PathBuf::from(receiver);
    let receiver_extension = path_receiver.extension().unwrap().to_str().unwrap().to_string();
    // Check that the image type of the receiver file is correct.
    let opt_receiver_type = ImageFormat::from_extension(receiver_extension);
    if opt_receiver_type.is_none() {
        return Err(format!("Invalid receiver file image type \"{}\".", receiver));
    }

    // Load the source image into memory.
    let image_data: DynamicImage = image::open(source).unwrap();

    let (nwidth, nheight) = image_data.dimensions();
    // if min_width > 0 || min_height > 0 {
    // 700x469 699x463 705x467 620x437
    // image01.png 1531x858 => image01_1024.jpg 1024x573
    // 1531÷858=1,784382284                     1024÷573=1,787085515
    // 1531 * x = 1024  x=1,495117188             // 1531 / 1,495117188 = 1023,999999658
    //  858 * y = 573   y=1,497382199             //  858 / 1,497382199 =  572,999999982
    // }
    // Delete the source image file.
    // let _ = fs::remove_file(source).await.unwrap();

    // Save the image from memory to the receiver.
    let result = image_data
        .resize_exact(nwidth, nheight, FilterType::Nearest)
        .save(receiver)
        .map_err(|err| err.to_string());

    match result {
        Ok(()) => Ok(receiver.to_string()),
        Err(err) => Err(err),
    }
}
