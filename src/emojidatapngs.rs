use std::{io::Cursor, path::Path};

use crate::{
    EmojiImage,
    provider::{ImageProvider, Provider},
};

pub struct EmojiDataPngs<'a> {
    dir: &'a Path,
}

impl<'a> EmojiDataPngs<'a> {
    pub fn new(dir: &'a Path) -> Self {
        Self { dir }
    }
}

impl ImageProvider for EmojiDataPngs<'_> {
    fn read_image(&self, emoji: &EmojiImage) -> Result<image::DynamicImage, crate::Error> {
        let mut path = self.dir.join(emoji.unified);
        path.set_extension("png");
        let data = fs_err::read(path).or_else(|e| match emoji.non_qualified {
            Some(nq) => {
                let mut path = self.dir.join(nq);
                path.set_extension("png");
                fs_err::read(path)
            }
            None => Err(e),
        })?;
        let mut reader = image::ImageReader::new(Cursor::new(data));
        reader.set_format(image::ImageFormat::Png);
        reader.decode().map_err(Into::into)
    }
}

impl Provider for EmojiDataPngs<'_> {
    fn transform(
        &self,
        options: &crate::Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), crate::Error> {
        self.transform_image(options, emoji, sizes)
    }
}
