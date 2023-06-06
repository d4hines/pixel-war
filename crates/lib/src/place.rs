use image::png::PngEncoder;
use image::{GenericImage, GenericImageView, Rgb, DynamicImage};
use std::io::Cursor;
use std::path::PathBuf;

use crate::message::{Content, PlacePixel, UserMessage};

#[derive(Debug)]
pub struct PlaceState {
    pub img: image::RgbImage,
    img_buf: Option<Vec<u8>>,
    path: PathBuf,
}

impl PlaceState {
    // TODO: make this a result
    pub fn set_pixel(&mut self, message: UserMessage) -> (bool, UserMessage) {
        let Content::PlacePixel(PlacePixel { x, y, color }) = message.inner().content;
        println!(
            "Setting pixel: {}, {}, rgb: {}, {}, {}",
            x, y, color[0], color[1], color[2]
        );
        let color = Rgb([color[0], color[1], color[2]]);
        let (width, height) = self.img.dimensions();

        if x < width && y < height {
            self.img.put_pixel(x, y, color);
            self.img_buf = None;
            // self.img.save(self.path.clone()).unwrap();
            (true, message)
        } else {
            (false, message)
        }
    }

    pub fn save(&mut self)  {
        self.img.save(self.path.clone()).unwrap();
    }

    pub fn new(path: PathBuf) -> Self {
        let bytes =
            std::fs::read(&path).unwrap_or(include_bytes!("./white_image.png").to_vec());
        Self::from_bytes(&bytes, path)
    }

    pub fn get_image_bytes(&mut self) -> Vec<u8> {
        if let Some(cached) = &self.img_buf {
            return cached.clone();
        }
        let img: DynamicImage = DynamicImage::ImageRgb8(self.img.clone());
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);

        let encoder = PngEncoder::new(&mut cursor);
        encoder
            .encode(
                img.as_bytes(),
                img.width(),
                img.height(),
                img.color(),
            )
            .unwrap();

        self.img_buf = Some(buf.clone());

        buf
    }

    pub fn from_bytes(bytes: &[u8], path: PathBuf) -> Self {
        let img = image::load_from_memory(bytes).unwrap();
        let img = img.to_rgb8(); 
        PlaceState {
            img,
            img_buf: None,
            path,
        }
    }
}
