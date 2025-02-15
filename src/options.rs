use std::path::PathBuf;

use crate::{EmojiImage, directories::Directories, optimize::Optimizer};

pub struct Options<'a> {
    pub resize: fast_image_resize::ResizeOptions,
    pub oxipng: oxipng::Options,
    pub zopfli: zopflipng::Options<'static>,
    pub optimizer: &'a dyn Optimizer,
    pub webp: libwebp_sys::WebPConfig,
    directories: Directories,
}

impl<'a> Options<'a> {
    pub fn new(directories: Directories, optimizer: &'a dyn Optimizer) -> Self {
        let resize = fast_image_resize::ResizeOptions::new().resize_alg(
            fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
        );

        let oxipng = oxipng::Options::max_compression();

        let mut zopfli = zopflipng::Options::new();
        zopfli
            .set_lossy_transparent(true)
            .set_num_iterations(15)
            .set_num_iterations_large(11)
            .set_filter_strategies(&[
                zopflipng::STRATEGY_ZERO,
                zopflipng::STRATEGY_PREDEFINED,
                zopflipng::STRATEGY_MIN_SUM,
                zopflipng::STRATEGY_ENTROPY,
            ]);

        let mut webp = libwebp_sys::WebPConfig::new_with_preset(
            libwebp_sys::WebPPreset::WEBP_PRESET_ICON,
            100.0,
        )
        .unwrap();
        webp.lossless = 1;
        webp.quality = 100.0;
        webp.method = 6;
        webp.image_hint = libwebp_sys::WebPImageHint::WEBP_HINT_GRAPH;
        webp.use_sharp_yuv = 1;

        Self {
            resize,
            oxipng,
            zopfli,
            optimizer,
            webp,
            directories,
        }
    }

    pub fn emoji_dir(&self, size: u32, emoji: &EmojiImage) -> PathBuf {
        self.directories.for_emoji(size, emoji)
    }
}
