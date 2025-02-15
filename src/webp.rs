use std::ops::Deref;

use crate::optimize::{OptimizableImage, Optimizer};

pub struct Webp;

impl Optimizer for Webp {
    fn optimize_fir(
        &self,
        options: &crate::Options,
        image: fast_image_resize::images::Image,
        path: &std::path::Path,
    ) -> Result<(), crate::Error> {
        optimize(options, image, path)
    }

    fn optimize_skia(
        &self,
        options: &crate::Options,
        image: tiny_skia::Pixmap,
        path: &std::path::Path,
    ) -> Result<(), crate::Error> {
        optimize(options, image, path)
    }
}

fn optimize(
    options: &crate::Options,
    image: impl OptimizableImage,
    path: &std::path::Path,
) -> Result<(), crate::Error> {
    let encoded = ::webp::Encoder::from_rgba(image.data(), image.width(), image.height())
        .encode_advanced(&options.webp)
        .map_err(crate::Error::Webp)?;
    fs_err::write(path, encoded.deref()).map_err(Into::into)
}
