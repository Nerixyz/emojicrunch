use usvg::Transform;

use crate::directories::Directories;
use crate::{EmojiImage, Options};
use crate::{Error, resize::resize};

pub trait Provider: Sync {
    fn transform(
        &self,
        dirs: &Directories,
        options: &Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), Error>;
}

pub trait ImageProvider: Sync {
    fn read_image(&self, emoji: &EmojiImage) -> Result<image::DynamicImage, Error>;

    fn transform_image(
        &self,
        dirs: &Directories,
        options: &Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), Error> {
        let base_image = self.read_image(emoji)?.into_rgba8();
        for &size in sizes {
            let resized = resize(&options.resize, &base_image, size)?;
            options
                .optimizer
                .optimize_fir(options, resized, &dirs.for_emoji(size, emoji))?;
        }
        Ok(())
    }
}

pub trait SvgProvider: Sync {
    fn read_svg(&self, emoji: &EmojiImage) -> Result<usvg::Tree, Error>;

    fn transform_svg(
        &self,
        dirs: &Directories,
        options: &Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), Error> {
        let svg = self.read_svg(emoji)?;
        let svg_size = svg.size().width().max(svg.size().height());
        for &size in sizes {
            let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
            let scale = (size as f32) / svg_size;
            let transform = Transform::from_scale(scale, scale);
            resvg::render(&svg, transform, &mut pixmap.as_mut());
            options
                .optimizer
                .optimize_skia(options, pixmap, &dirs.for_emoji(size, emoji))?;
        }
        Ok(())
    }
}
