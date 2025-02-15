use std::{collections::HashMap, fmt::Display, iter, path::PathBuf};

use apple::AppleFont;
use clap::Parser;
use directories::Directories;
use emojidatapngs::EmojiDataPngs;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use notoemoji::Notoemoji;
use optimize::{Optimizer, Oxipng, Zopflipng};
use provider::Provider;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod apple;
mod directories;
mod emojidatapngs;
mod error;
mod notoemoji;
mod optimize;
mod options;
mod provider;
mod resize;
mod twemoji;
mod webp;

pub use error::Error;
pub use options::Options;
use twemoji::Twemoji;
use webp::Webp;

#[derive(serde::Deserialize)]
struct Emoji {
    pub short_name: String,
    #[serde(deserialize_with = "deser_lower")]
    pub unified: String,
    #[serde(deserialize_with = "deser_lower_opt")]
    pub non_qualified: Option<String>,
    #[serde(default)]
    pub skin_variations: HashMap<String, SkinVariation>,
}

#[derive(serde::Deserialize)]
struct SkinVariation {
    #[serde(deserialize_with = "deser_lower_opt")]
    pub non_qualified: Option<String>,
    #[serde(deserialize_with = "deser_lower")]
    pub unified: String,
}

pub struct EmojiImage<'a> {
    pub unified: &'a str,
    pub non_qualified: Option<&'a str>,
    pub short_name: &'a str,
}

fn transform_for(
    provider: &impl Provider,
    options: &Options,
    emojis: &[EmojiImage],
    sizes: &[u32],
) {
    let it = emojis
        .par_iter()
        .progress_with_style(ProgressStyle::with_template("{bar} {pos:>7}/{len:7} {eta}").unwrap())
        .filter_map(|it| match provider.transform(options, it, sizes) {
            Err(e) => Some(format!(
                ":{}: ({}) failed: {}",
                it.short_name, it.unified, e
            )),
            Ok(_) => None,
        })
        .collect_vec_list();
    for e in it.into_iter().flatten() {
        eprintln!("{e}");
    }
}

#[derive(clap::Parser)]
#[command(name = "emojicrunch", about = "Resize and render emojis")]
struct Args {
    /// Sets the path to the Apple font
    ///
    /// If set, emojis will be extracted from that font.
    #[arg(long, value_name = "PATH")]
    apple_font: Option<PathBuf>,

    #[arg(short, default_value = "build")]
    output_dir: PathBuf,

    /// Root directory of the emoji-data repository.
    ///
    /// Submodules must be checked out
    #[arg(long, default_value = "emoji-data")]
    emoji_data_root: PathBuf,

    /// Use zopfli to compress the PNGs
    ///
    /// This is significantly slower but results in smaller PNGs
    #[arg(long, default_value = "false")]
    use_zopfli: bool,

    /// Produce WEBPs
    #[arg(long, default_value = "false")]
    webp: bool,

    /// The size(s) to scale/render to. Accepts multiple arguments.
    ///
    /// Defaults to [22, 22 * 2, 22 * 4]
    #[arg(long)]
    size: Vec<u32>,

    /// List of vendors to build for
    #[arg(value_enum)]
    vendors: Vec<Vendor>,
}

#[derive(Clone, PartialEq, Eq, clap::ValueEnum)]
enum Vendor {
    Apple,
    Twitter,
    Google,
    Facebook,
}

impl Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vendor::Apple => write!(f, "apple"),
            Vendor::Twitter => write!(f, "twitter"),
            Vendor::Google => write!(f, "google"),
            Vendor::Facebook => write!(f, "facebook"),
        }
    }
}

fn main() {
    let mut args = Args::parse();
    args.vendors.dedup();
    args.size.dedup();

    if args.size.is_empty() {
        args.size = vec![22, 22 * 2, 22 * 4];
    }
    if args.vendors.is_empty() {
        args.vendors = vec![
            Vendor::Apple,
            Vendor::Twitter,
            Vendor::Google,
            Vendor::Facebook,
        ];
    }

    println!("Reading emoji.json");
    let emojis: Vec<Emoji> =
        serde_json::from_slice(&fs_err::read(args.emoji_data_root.join("emoji.json")).unwrap())
            .unwrap();
    let images: Vec<EmojiImage<'_>> = emojis
        .iter()
        .flat_map(|it| {
            iter::once(EmojiImage {
                short_name: &it.short_name,
                non_qualified: it.non_qualified.as_deref(),
                unified: &it.unified,
            })
            .chain(it.skin_variations.values().map(|v| EmojiImage {
                short_name: &it.short_name,
                non_qualified: v.non_qualified.as_deref(),
                unified: &v.unified,
            }))
        })
        .collect();

    let optimizer: Box<dyn Optimizer> = if args.webp {
        Box::new(Webp)
    } else if args.use_zopfli {
        Box::new(Zopflipng)
    } else {
        Box::new(Oxipng)
    };
    let extension = if args.webp { "webp" } else { "png" };

    for vendor in args.vendors {
        let name = vendor.to_string();
        println!("Processing {name}...");

        let directories = Directories::for_provider("build", &name, extension);
        directories.create_sizes(&args.size).unwrap();
        let options = Options::new(directories, optimizer.as_ref());

        match vendor {
            Vendor::Apple => match args.apple_font {
                Some(ref font) => {
                    let font = std::fs::read(font).unwrap();
                    let apple = AppleFont::new(&font, 0).unwrap();
                    transform_for(&apple, &options, &images, &args.size);
                }
                None => {
                    let root = args.emoji_data_root.join("img-apple-160");
                    let apple = EmojiDataPngs::new(&root);
                    transform_for(&apple, &options, &images, &args.size);
                }
            },
            Vendor::Twitter => {
                let root = args
                    .emoji_data_root
                    .join("build/twitter/twemoji/assets/svg");
                let twemoji = Twemoji::new(&root);
                transform_for(&twemoji, &options, &images, &args.size);
            }
            Vendor::Google => {
                let root = args.emoji_data_root.join("build/google/noto-emoji/svg");
                let noto = Notoemoji::new(&root);
                transform_for(&noto, &options, &images, &args.size);
            }
            Vendor::Facebook => {
                let root = args.emoji_data_root.join("img-facebook-96");
                let facebook = EmojiDataPngs::new(&root);
                transform_for(&facebook, &options, &images, &args.size);
            }
        }
    }
}

fn deser_lower<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl serde::de::Visitor<'_> for Visitor {
        type Value = String;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.to_ascii_lowercase())
        }
    }

    d.deserialize_str(Visitor)
}

fn deser_lower_opt<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl<'deinner> serde::de::Visitor<'deinner> for Visitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a string")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'deinner>,
        {
            deserializer.deserialize_str(Visitor)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_ascii_lowercase()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    d.deserialize_option(Visitor)
}
