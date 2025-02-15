use std::path::Path;

use crate::{
    EmojiImage,
    provider::{Provider, SvgProvider},
};

pub struct Twemoji<'a> {
    svg_dir: &'a Path,
    options: usvg::Options<'static>,
}

impl<'a> Twemoji<'a> {
    pub fn new(svg_dir: &'a Path) -> Self {
        Self {
            svg_dir,
            options: usvg::Options::default(),
        }
    }
}

impl SvgProvider for Twemoji<'_> {
    fn read_svg(&self, emoji: &EmojiImage) -> Result<usvg::Tree, crate::Error> {
        let mut path = self.svg_dir.join(strip_zeroes(emoji.unified));
        path.set_extension("svg");

        let data = fs_err::read(path).or_else(|e| match emoji.non_qualified {
            Some(nq) => {
                let mut path = self.svg_dir.join(strip_zeroes(nq));
                path.set_extension("svg");
                fs_err::read(path)
            }
            None => Err(e),
        })?;
        usvg::Tree::from_data(&data, &self.options).map_err(Into::into)
    }
}

impl Provider for Twemoji<'_> {
    fn transform(
        &self,
        dirs: &crate::directories::Directories,
        options: &crate::Options,
        emoji: &EmojiImage,
        sizes: &[u32],
    ) -> Result<(), crate::Error> {
        self.transform_svg(dirs, options, emoji, sizes)
    }
}

fn strip_zeroes(mut s: &str) -> &str {
    while let Some(n) = s.strip_prefix('0') {
        s = n;
    }
    s
}
