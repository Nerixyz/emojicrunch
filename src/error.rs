#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Emoji isn't available for this provider")]
    NoEmoji,
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Oxipng: {0}")]
    Oxipng(#[from] oxipng::PngError),
    #[error("Resize: {0}")]
    Resize(#[from] fast_image_resize::ResizeError),
    #[error("Usvg: {0}")]
    Usvg(#[from] usvg::Error),
    #[error("Zopfli: {0}")]
    Zopfli(i32),
    #[error("Webp: {0:?}")]
    Webp(::webp::WebPEncodingError),
}
