use std::{ffi::c_void, path::Path};

use image::{DynamicImage, GenericImageView};

#[derive(Debug)]
pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub unsafe fn new(image: DynamicImage, flip_vertical: bool) -> Self {
        let image = if flip_vertical { image.flipv() } else { image };

        let width = image.width();
        let height = image.width();

        let (data, gl_format) = match image {
            DynamicImage::ImageRgb8(image) => (image.into_raw(), gl::RGB),
            DynamicImage::ImageRgba8(image) => (image.into_raw(), gl::RGBA),
            _ => unimplemented!(),
        };

        let mut id = 0;
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);
        gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        gl::TextureStorage2D(id, 1, gl::RGBA8, width as i32, height as i32);
        gl::TextureSubImage2D(
            id,
            0,
            0,
            0,
            width as i32,
            height as i32,
            gl_format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );

        Self { id, width, height }
    }

    pub unsafe fn from_path<P>(path: P, flip_vertical: bool) -> Self
    where
        P: AsRef<Path>,
    {
        let image = image::open(path).expect("Couldn't open image");
        Self::new(image, flip_vertical)
    }

    pub unsafe fn bind_to_unit(&self, unit: u32) {
        gl::BindTextureUnit(unit, self.id);
    }
}
