use std::path::Path;

use image::{ImageEncoder, codecs::png::PngEncoder};

use crate::Error;

pub trait OptimizableImage {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn into_data(self) -> Vec<u8>;
    fn into_png(self) -> Vec<u8>;
    fn data(&self) -> &[u8];
}

pub trait Optimizer: Sync {
    fn optimize_fir(
        &self,
        options: &crate::Options,
        image: fast_image_resize::images::Image,
        path: &Path,
    ) -> Result<(), Error>;
    fn optimize_skia(
        &self,
        options: &crate::Options,
        image: tiny_skia::Pixmap,
        path: &Path,
    ) -> Result<(), Error>;
}

pub struct Oxipng;
pub struct Zopflipng;

impl Oxipng {
    fn optimize(
        options: &oxipng::Options,
        image: impl OptimizableImage,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let raw = oxipng::RawImage::new(
            image.width(),
            image.height(),
            oxipng::ColorType::RGBA,
            oxipng::BitDepth::Eight,
            image.into_data(),
        )?;
        let png = raw.create_optimized_png(options)?;
        fs_err::write(path, png).map_err(Into::into)
    }
}

impl Optimizer for Oxipng {
    fn optimize_fir(
        &self,
        options: &crate::Options,
        image: fast_image_resize::images::Image,
        path: &Path,
    ) -> Result<(), Error> {
        Self::optimize(&options.oxipng, image, path)
    }

    fn optimize_skia(
        &self,
        options: &crate::Options,
        image: tiny_skia::Pixmap,
        path: &Path,
    ) -> Result<(), Error> {
        Self::optimize(&options.oxipng, image, path)
    }
}

impl Zopflipng {
    fn optimize(
        options: &zopflipng::Options,
        image: impl OptimizableImage,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let data = image.into_png();
        let out = zopflipng::optimize(&data, options).map_err(Error::Zopfli)?;
        fs_err::write(path, out).map_err(Into::into)
    }
}

impl Optimizer for Zopflipng {
    fn optimize_fir(
        &self,
        options: &crate::Options,
        image: fast_image_resize::images::Image,
        path: &Path,
    ) -> Result<(), Error> {
        Self::optimize(&options.zopfli, image, path)
    }

    fn optimize_skia(
        &self,
        options: &crate::Options,
        image: tiny_skia::Pixmap,
        path: &Path,
    ) -> Result<(), Error> {
        Self::optimize(&options.zopfli, image, path)
    }
}

impl<'a> OptimizableImage for fast_image_resize::images::Image<'a> {
    fn width(&self) -> u32 {
        fast_image_resize::images::Image::width(&self)
    }

    fn height(&self) -> u32 {
        fast_image_resize::images::Image::height(&self)
    }

    fn into_data(self) -> Vec<u8> {
        self.into_vec()
    }

    fn into_png(self) -> Vec<u8> {
        let mut out = Vec::new();
        assert_eq!(self.pixel_type(), fast_image_resize::PixelType::U8x4);
        PngEncoder::new(&mut out)
            .write_image(
                self.buffer(),
                self.width(),
                self.height(),
                image::ExtendedColorType::Rgba8,
            )
            .ok();
        out
    }

    fn data(&self) -> &[u8] {
        self.buffer()
    }
}

impl OptimizableImage for tiny_skia::Pixmap {
    fn width(&self) -> u32 {
        tiny_skia::Pixmap::width(&self)
    }

    fn height(&self) -> u32 {
        tiny_skia::Pixmap::height(&self)
    }

    fn into_data(mut self) -> Vec<u8> {
        // png mandates unmultiplied alpha
        // https://www.w3.org/TR/png/#6AlphaRepresentation
        // ... but tiny-skia hides PremultipliedColorU8::from_rgba_unchecked

        for pixel in bytemuck::cast_slice_mut::<_, FakePremultipliedColorU8>(self.data_mut()) {
            pixel.demultiply();
        }

        self.take()
    }

    fn into_png(self) -> Vec<u8> {
        let width = self.width();
        let height = self.height();
        let data = self.into_data();
        let mut out = Vec::new();
        PngEncoder::new(&mut out)
            .write_image(&data, width, height, image::ExtendedColorType::Rgba8)
            .ok();
        out
    }

    fn data(&self) -> &[u8] {
        self.data()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct FakePremultipliedColorU8([u8; 4]);

// same as PremultipliedColorU8
unsafe impl bytemuck::Zeroable for FakePremultipliedColorU8 {}
unsafe impl bytemuck::Pod for FakePremultipliedColorU8 {}

impl FakePremultipliedColorU8 {
    pub fn red(&self) -> u8 {
        self.0[0]
    }
    pub fn green(&self) -> u8 {
        self.0[1]
    }
    pub fn blue(&self) -> u8 {
        self.0[2]
    }
    pub fn alpha(&self) -> u8 {
        self.0[3]
    }
    pub fn demultiply(&mut self) {
        let alpha = self.alpha();
        if alpha != u8::MAX {
            let a = alpha as f64 / 255.0;
            self.0 = [
                (self.red() as f64 / a + 0.5) as u8,
                (self.green() as f64 / a + 0.5) as u8,
                (self.blue() as f64 / a + 0.5) as u8,
                alpha,
            ]
        }
    }
}
