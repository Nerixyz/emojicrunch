use std::{env, path::PathBuf};

fn main() {
    cc::Build::new()
        .files(&[
            "zopfli/src/zopfli/blocksplitter.c",
            "zopfli/src/zopfli/cache.c",
            "zopfli/src/zopfli/deflate.c",
            "zopfli/src/zopfli/gzip_container.c",
            "zopfli/src/zopfli/hash.c",
            "zopfli/src/zopfli/katajainen.c",
            "zopfli/src/zopfli/lz77.c",
            "zopfli/src/zopfli/squeeze.c",
            "zopfli/src/zopfli/tree.c",
            "zopfli/src/zopfli/util.c",
            "zopfli/src/zopfli/zlib_container.c",
            "zopfli/src/zopfli/zopfli_lib.c",
        ])
        .compile("zopfli");
    cc::Build::new()
        .cpp(true)
        .files(&[
            "zopfli/src/zopflipng/lodepng/lodepng.cpp",
            "zopfli/src/zopflipng/lodepng/lodepng_util.cpp",
            "zopfli/src/zopflipng/zopflipng_lib.cc",
        ])
        .compile("zopflipng");

    let bindings = bindgen::Builder::default()
        .header("zopfli/src/zopflipng/zopflipng_lib.h")
        .allowlist_item("CZopfli.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
