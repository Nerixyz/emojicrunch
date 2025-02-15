use std::path::{Path, PathBuf};

use crate::{
    EmojiImage,
    provider::{Provider, SvgProvider},
};

pub struct Notoemoji<'a> {
    svg_dir: &'a Path,
    flag_dir: PathBuf,
    options: usvg::Options<'static>,
}

impl<'a> Notoemoji<'a> {
    pub fn new(svg_dir: &'a Path) -> Self {
        let flag_dir = svg_dir
            .parent()
            .unwrap()
            .join("third_party/region-flags/waved-svg");
        Self {
            svg_dir,
            flag_dir,
            options: usvg::Options::default(),
        }
    }
}

fn is_flag(s: &str) -> bool {
    matches!(
        s,
        "cn" | "de" | "es" | "fr" | "gb" | "it" | "jp" | "kr" | "ru" | "us"
    ) || s.starts_with("flag-")
}

impl SvgProvider for Notoemoji<'_> {
    fn read_svg(&self, emoji: &EmojiImage) -> Result<usvg::Tree, crate::Error> {
        let file = format!(
            "emoji_u{}.svg",
            emoji.non_qualified.unwrap_or(emoji.unified)
        );
        let file = file.replace('-', "_");
        let path = if is_flag(emoji.short_name) {
            &self.flag_dir
        } else {
            self.svg_dir
        }
        .join(file);

        let data = fs_err::read(path)?;
        usvg::Tree::from_data(&data, &self.options).map_err(Into::into)
    }
}

impl Provider for Notoemoji<'_> {
    fn transform(
        &self,
        options: &crate::Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), crate::Error> {
        self.transform_svg(options, emoji, sizes)
    }
}
