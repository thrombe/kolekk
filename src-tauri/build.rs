use std::{
    fs::{create_dir_all, remove_dir, remove_file, File},
    io::{Read, Write},
    path::Path,
};

use kolekk_types::{
    api::{
        tachidesk::{Chapter, Extension, Manga, Source, ExtensionAction},
        tmdb::{
            AllInfo, AltTitles, ExternalIDs, ExternalIdSearchResult, Genre, ImageInfo, Images,
            ListResults, Movie, MovieListResult, MultiSearchResult, Season, Title, Tv,
            TvListResult,
        },
    },
    Bookmark, ByteArrayFile, DragDropPaste, FilderKind, Group, Image, Object, Tag, TS,
};

fn main() {
    println!("cargo:rerun-if-changed=./crates/kolekk-types/src/lib.rs");

    let cache_dir = Path::new("../cache/ts_bindings");
    let output_file = Path::new("../src/rs_bindings.ts");

    create_dir_all(cache_dir).unwrap();
    let mut contents = String::new();

    macro_rules! export {
        ($($t:ty), *  $(,)?) => {
            $(
                let name = format!("{}.ts", stringify!($t));
                let mut ttype = String::new();
                let path = cache_dir.to_path_buf().join(name);
                <$t as TS>::export_to(&path).unwrap();
                let mut file = File::open(&path).unwrap();
                file.read_to_string(&mut ttype).unwrap();
                for line in ttype.lines() {
                    if line.starts_with("import") {
                        contents += &format!("// {}\n", line);
                    } else {
                        contents += &format!("{}\n", line);
                    }
                }
                let _ = remove_file(&path);
            )*

        };
    }

    // TODO: no 2 types can have the same name T-T
    export!(
        Bookmark,
        Image,
        DragDropPaste<()>,
        ByteArrayFile,
        FilderKind,
        Object,
        Tag,
        Group,
    );
    export!(
        AllInfo<()>,
        AltTitles,
        ExternalIDs,
        ExternalIdSearchResult,
        Images,
        ImageInfo,
        ListResults<()>,
        Movie,
        MovieListResult,
        MultiSearchResult,
        TvListResult,
        Tv,
        Title,
        Genre,
        Season,
    );
    export!(
        Extension,
        Manga,
        Chapter,
        Source,
        ExtensionAction,
    );

    remove_dir(cache_dir).unwrap();

    let mut output = File::create(output_file).unwrap();
    write!(output, "{contents}").unwrap();

    tauri_build::build()
}
