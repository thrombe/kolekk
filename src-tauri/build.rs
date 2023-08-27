use std::{
    fs::{create_dir_all, remove_dir, remove_file, File},
    io::{Read, Write},
    path,
};

use kolekk_types::{
    api::{
        lastfm::{
            AlbumInfo, AlbumListResult, AlbumTrack, AlbumTrackArtist, ArtistInfo,
            ArtistInfoSimilar, ArtistListResult, InfoQuery, InfoStats, InfoWiki, LfmImage,
            LfmImageSize, LfmTag, Link, SearchQuery, SearchResultsOk, SimilarTrack, TrackInfo,
            TrackInfoAlbum, TrackInfoArtist, TrackListResult,
        },
        tachidesk::{
            About, Chapter, Extension, ExtensionAction, Manga, MangaListPage, MangaSource,
            MetaValue, SelectableItem, SortFilter, SourceFilter,
        },
        tmdb::{
            AllInfo, AltTitles, ExternalIDs, ExternalIdSearchResult, Genre, ImageInfo, Images,
            ListResults, Movie, MovieListResult, MultiSearchResult, Season, Title, Tv,
            TvListResult,
        },
    },
    objects::{
        Bookmark, Content, Fields, Group, Image, Indexed, Meta, Notes, SearchableEntry, Tag,
        Taggable, TypeFacet,
    },
    utility::{
        BasePath, ByteArrayFile, DdpInfo, DirFiles, DragDropData, DragDropPaste, Path, Source,
        ThumbnailSize,
    },
    FilderKind, TS,
};

fn main() {
    println!("cargo:rerun-if-changed=./crates/kolekk-types/src/lib.rs");

    let cache_dir = path::Path::new("../cache/ts_bindings");
    let output_file = path::Path::new("../src/rs_bindings.ts");

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
    // have each call to export macro output stuff in a different file
    export!(
        Image,
        Bookmark,
        Content,
        Notes,
        Tag,
        Group,
        Meta<(), ()>,
        Taggable<()>,
        Fields,
        TypeFacet,
        Path,
        BasePath,
        Source,
        ThumbnailSize,
        SearchableEntry<()>,
        Indexed,
        DragDropPaste<()>,
        DragDropData<()>,
        DdpInfo<()>,
        DirFiles,
        ByteArrayFile,
        FilderKind,
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
        MangaSource,
        ExtensionAction,
        MangaListPage,
        About,
        MetaValue,
        SourceFilter,
        SortFilter,
        SelectableItem,
    );
    export!(
        InfoQuery<()>,
        SearchResultsOk<()>,
        SearchQuery,
        LfmImageSize,
        LfmImage,
        ArtistListResult,
        AlbumListResult,
        TrackListResult,
        AlbumTrackArtist,
        AlbumTrack,
        AlbumInfo<(), ()>,
        LfmTag,
        ArtistInfo<(), (), ()>,
        InfoStats,
        TrackInfo<()>,
        Link,
        InfoWiki<()>,
        TrackInfoAlbum,
        TrackInfoArtist,
        SimilarTrack,
        ArtistInfoSimilar,
    );

    remove_dir(cache_dir).unwrap();

    let mut output = File::create(output_file).unwrap();
    write!(output, "{contents}").unwrap();

    tauri_build::build()
}
