use std::io::Cursor;

use rustybuzz::{ShapePlan, UnicodeBuffer, shape_with_plan};
use ttf_parser::GlyphId;

use crate::provider::{ImageProvider, Provider};
use crate::{EmojiImage, Error};

fn parse_hex<'a>(hex: &str, buffer: &'a mut [u8]) -> &'a mut str {
    let mut size = 0;
    for c in hex.split('-') {
        let c = u32::from_str_radix(c, 16)
            .ok()
            .and_then(char::from_u32)
            .unwrap();
        size += c.encode_utf8(&mut buffer[size..]).len();
    }
    // this will always work
    std::str::from_utf8_mut(&mut buffer[..size]).unwrap()
}

pub struct AppleFont<'a> {
    face: rustybuzz::Face<'a>,
    plan: rustybuzz::ShapePlan,
    strike: ttf_parser::sbix::Strike<'a>,
}

impl ImageProvider for AppleFont<'_> {
    fn read_image(&self, emoji: &EmojiImage) -> Result<image::DynamicImage, Error> {
        let mut unicode_buffer = UnicodeBuffer::new();
        let mut utf8_buffer = [0u8; 64];
        unicode_buffer.push_str(parse_hex(emoji.unified, &mut utf8_buffer));
        let glyphs = shape_with_plan(&self.face, &self.plan, unicode_buffer);
        if glyphs.is_empty() {
            return Err(Error::NoEmoji);
        }

        if glyphs.len() > 1 {
            let mut img = None;
            for glyph in glyphs.glyph_infos() {
                let raster_img = self
                    .strike
                    .get(GlyphId(glyph.glyph_id as u16))
                    .ok_or_else(|| Error::NoEmoji)?;

                let mut reader = image::ImageReader::new(Cursor::new(raster_img.data));
                reader.set_format(image::ImageFormat::Png);
                let decoded = reader.decode()?;
                if let Some(ref mut bg) = img {
                    image::imageops::overlay(bg, &decoded, 0, 0);
                } else {
                    img = Some(decoded);
                }
            }

            img.ok_or_else(|| Error::NoEmoji)
        } else {
            let id = GlyphId(glyphs.glyph_infos()[0].glyph_id as u16);
            let img = self.strike.get(id).ok_or_else(|| Error::NoEmoji)?;
            let mut reader = image::ImageReader::new(Cursor::new(img.data));
            reader.set_format(image::ImageFormat::Png);
            reader.decode().map_err(Into::into)
        }
    }
}

impl Provider for AppleFont<'_> {
    fn transform(
        &self,
        dirs: &crate::directories::Directories,
        options: &crate::Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), Error> {
        self.transform_image(dirs, options, emoji, sizes)
    }
}

impl<'a> AppleFont<'a> {
    pub fn new(data: &'a [u8], face_index: u32) -> Option<Self> {
        let face = rustybuzz::Face::from_slice(&data, face_index)?;
        let plan = ShapePlan::new(&face, rustybuzz::Direction::LeftToRight, None, None, &[]);
        let strike = face
            .tables()
            .sbix?
            .strikes
            .into_iter()
            .max_by_key(|x| x.pixels_per_em)?;
        Some(Self { face, plan, strike })
    }
}
