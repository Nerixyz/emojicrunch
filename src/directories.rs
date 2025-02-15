use std::path::{Path, PathBuf};

use crate::EmojiImage;

pub struct Directories {
    base_dir: PathBuf,
    extension: &'static str,
}

impl Directories {
    pub fn new(base_dir: PathBuf, extension: &'static str) -> Self {
        Self {
            base_dir,
            extension,
        }
    }

    pub fn for_provider(
        base_dir: impl AsRef<Path>,
        provider: impl AsRef<Path>,
        extension: &'static str,
    ) -> Self {
        Self::new(base_dir.as_ref().join(provider.as_ref()), extension)
    }

    pub fn create_sizes(&self, sizes: &[u32]) -> std::io::Result<()> {
        for size in sizes {
            fs_err::create_dir_all(self.base_dir.join(size.to_string()))?;
        }
        Ok(())
    }

    pub fn for_emoji(&self, size: u32, emoji: &EmojiImage) -> PathBuf {
        self.base_dir
            .join(format!("{size}/{}.{}", emoji.unified, self.extension))
    }
}
