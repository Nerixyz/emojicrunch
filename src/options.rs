use crate::optimize::{Optimizer, Oxipng, Zopflipng};

pub struct Options {
    pub resize: fast_image_resize::ResizeOptions,
    pub oxipng: oxipng::Options,
    pub zopfli: zopflipng::Options<'static>,
    pub optimizer: Box<dyn Optimizer>,
}

impl Default for Options {
    fn default() -> Self {
        let resize = fast_image_resize::ResizeOptions::new().resize_alg(
            fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
        );

        let oxipng = oxipng::Options::max_compression();
        // oxipng.deflate = oxipng::Deflaters::Zopfli {
        //     iterations: 5.try_into().unwrap(),
        // };

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

        Self {
            resize,
            oxipng,
            zopfli,
            optimizer: Box::new(Oxipng),
        }
    }
}

impl Options {
    pub fn use_zopfli(&mut self) {
        self.optimizer = Box::new(Zopflipng)
    }
}
