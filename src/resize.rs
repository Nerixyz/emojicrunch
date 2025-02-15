use fast_image_resize::{PixelType, Resizer, images::TypedImageRef};
use image::EncodableLayout;

use crate::Error;

pub fn resize(
    options: &fast_image_resize::ResizeOptions,
    image: &image::RgbaImage,
    size: u32,
) -> Result<fast_image_resize::images::Image<'static>, Error> {
    let mut dst_image =
        fast_image_resize::images::Image::new(size, size, fast_image_resize::PixelType::U8x4);

    Resizer::new().resize(&ImageWrap(image), &mut dst_image, options)?;

    Ok(dst_image)
}

struct ImageWrap<'a>(&'a image::RgbaImage);

impl fast_image_resize::IntoImageView for ImageWrap<'_> {
    fn pixel_type(&self) -> Option<PixelType> {
        Some(PixelType::U8x4)
    }

    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn image_view<P: fast_image_resize::PixelTrait>(
        &self,
    ) -> Option<impl fast_image_resize::ImageView<Pixel = P>> {
        if P::pixel_type() != PixelType::U8x4 {
            return None;
        }

        TypedImageRef::<P>::from_buffer(self.0.width(), self.0.height(), self.0.as_bytes()).ok()
    }
}
