use std::{
    fs::{create_dir_all, remove_dir, remove_file, File},
    io::{Read, Write},
    path::Path,
};

use kolekk_types::{Bookmark, Image, TS};

fn main() {
    println!("cargo:rerun-if-changed=./crates/kolekk-types/src/lib.rs");

    let cache_dir = Path::new("../cache/ts_bindings");
    let output_file = Path::new("../src/rs_bindings.ts");

    create_dir_all(cache_dir).unwrap();
    let mut contents = String::new();

    macro_rules! export {
        ($($t:ty), *) => {
            $(
                let name = format!("{}.ts", stringify!($t));
                let path = cache_dir.to_path_buf().join(name);
                <$t as TS>::export_to(&path).unwrap();
                let mut file = File::open(&path).unwrap();
                file.read_to_string(&mut contents).unwrap();
                let _ = remove_file(&path);
            )*

        };
    }

    export!(Bookmark, Image);

    remove_dir(cache_dir).unwrap();

    let mut output = File::create(output_file).unwrap();
    write!(output, "{contents}").unwrap();

    tauri_build::build()
}
