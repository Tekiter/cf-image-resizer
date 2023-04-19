use std::io::Cursor;

use image::{io::Reader as ImageReader, DynamicImage, ImageOutputFormat};

pub fn read_image(bytes: Vec<u8>) -> DynamicImage {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("This will never fail using Cursor");

    let img = reader.decode().expect("should decoded");

    img
}

pub fn write_image(image: DynamicImage) -> Result<Vec<u8>, ()> {
    let outbuf = vec![];
    let mut cursor = Cursor::new(outbuf);
    image
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .map_err(|_| ())?;

    let result = cursor.get_ref().to_vec();

    Ok(result)
}
