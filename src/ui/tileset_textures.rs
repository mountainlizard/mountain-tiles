use crate::data::tilesets::{Tileset, TilesetMode};
use camino::Utf8PathBuf;
use egui::{
    Context, ImageSource, SizeHint, TextureOptions,
    emath::OrderedFloat,
    load::{LoadError, TexturePoll},
};
use std::fmt::Display;

const MOUNTAIN_TILES: ImageSource<'static> =
    egui::include_image!("../../assets/mountain-tiles.png");

const MISSING_PALETTE: ImageSource<'static> =
    egui::include_image!("../../assets/missing_palette.png");
const MISSING_IMAGE: ImageSource<'static> = egui::include_image!("../../assets/missing_image.png");
const MISSING_TILESET: ImageSource<'static> = egui::include_image!("../../assets/missing_grid.png");

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ErrorTexture {
    MissingPalette,
    MissingImage,
    MissingTileset,
}

#[derive(Debug, Clone)]
pub enum TextureSource {
    Builtin,
    File { base_dir: Option<Utf8PathBuf> },
}

impl Default for TextureSource {
    fn default() -> Self {
        TextureSource::File { base_dir: None }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TilesetTextures {
    source: TextureSource,
}

pub enum PathStatus {
    ExistsAsFile,
    ExistsNotFile,
    DoesNotExist,
    Errored(String),
}

pub struct TilesetError {
    source: TextureSource,
    tileset_path: Utf8PathBuf,
    resolved_path: Utf8PathBuf,
    path_status: PathStatus,
    load_error: LoadError,
}

impl TilesetError {
    pub fn path_notes(&self) -> String {
        match &self.source {
            TextureSource::Builtin => "".to_string(),
            TextureSource::File { base_dir } => {
                if self.tileset_path.is_absolute() {
                    format!(
                        "Image has absolute path '{}', is this correct for your system? You might want to use a relative path.",
                        self.tileset_path
                    )
                } else {
                    match base_dir {
                        None => format!(
                            "Image has relative path '{}', but project has not been saved, try saving.",
                            self.tileset_path
                        ),
                        Some(base_dir) => format!(
                            "Image is at relative path '{}', project saved at '{}'.",
                            self.tileset_path, base_dir
                        ),
                    }
                }
            }
        }
    }
}

impl Display for TilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            TextureSource::Builtin => write!(
                f,
                "Missing builtin tileset {}, please report software issue",
                self.tileset_path
            ),
            TextureSource::File { base_dir: _ } => match &self.path_status {
                PathStatus::ExistsAsFile => write!(
                    f,
                    "Image file at '{}' failed to load ({}). If you have just created the file, try reloading the software (this is a known issue, we aim to improve this in future).",
                    self.resolved_path, self.load_error
                ),
                PathStatus::ExistsNotFile => write!(
                    f,
                    "Image expected at '{}', but is not a file (may be a directory?)",
                    self.resolved_path
                ),
                PathStatus::DoesNotExist => write!(
                    f,
                    "Image not found at '{}'.\n{}",
                    self.resolved_path,
                    self.path_notes()
                ),
                PathStatus::Errored(e) => write!(
                    f,
                    "Image at '{}' could not be loaded, error checking file status ({})",
                    self.resolved_path, e
                ),
            },
        }
    }
}

impl TilesetTextures {
    const SIZE_HINT: SizeHint = SizeHint::Scale(OrderedFloat(1.0));

    fn path_status(path: &Utf8PathBuf) -> PathStatus {
        match path.try_exists() {
            Ok(true) => match path.metadata() {
                Ok(metadata) => {
                    if metadata.is_file() {
                        PathStatus::ExistsAsFile
                    } else {
                        PathStatus::ExistsNotFile
                    }
                }
                Err(e) => PathStatus::Errored(e.to_string()),
            },
            Ok(false) => PathStatus::DoesNotExist,
            Err(e) => PathStatus::Errored(e.to_string()),
        }
    }

    fn path_for_tileset_from_base_dir(
        &self,
        base_dir: &Option<Utf8PathBuf>,
        tileset: &Tileset,
    ) -> Utf8PathBuf {
        if let Some(base_dir) = base_dir {
            let mut path = base_dir.clone();
            path.push(tileset.path.clone());
            path
        } else {
            tileset.path.clone()
        }
    }

    pub fn uri_for_path(path: &Utf8PathBuf, mode: &TilesetMode) -> String {
        let mode_json = serde_json::to_string(mode).unwrap_or("\"Direct\"".to_string());

        // We use a somewhat odd "uri" format here - we use a scheme of "tileset://"
        // to allow us to include the tileset mode as json, then another "//" to separate
        // this from a final section that is more or less a file uri.
        //
        // However note that the file uri will go to the `egui_extras` `FileLoader` loader,
        // and this doesn't quite parse standard file uris, since it doesn't expect
        // percent encoding, so we can't use say the `Url` crate. Instead we just tailor
        // the format for what `FileLoader` expects.
        //
        // In egui 0.36 the code is at:
        // https://github.com/emilk/egui/blob/a4edc4a93241782d5f0c0864ab052c9d1244cd98/crates/egui_extras/src/loaders/file_loader.rs#L32
        //
        // The uri must start with "file://", and this will be stripped
        // by `FileLoader`.
        //
        // The rest of the string is parsed as a path, but this is slightly different
        // on windows and other platforms.
        //
        // On non-windows platforms, the rest of the string is used directly as a path,
        // so we can just include it unaltered. This goes from a path like "/etc/fstab" to
        // "file:///etc/fstab", which matches what the `Url` crate does, and
        // is given as an example on wikipedia. Note that if there are spaces, these
        // are not percent-encoded.
        //
        // On windows we need to make it a local file path by
        // prepending an additional "/" to the file path, this will then be detected
        // and stripped as a prefix by the windows-specific egui code, so it gets the
        // original path back. This makes the uri compliant according to wikipedia.
        // So for example we would go from "c:/WINDOWS/clock.avi" to
        // "file:///c:/WINDOWS/clock.avi" (note the extra "/"), again matching
        // an example on wikipedia. Note this is necessary because otherwise we
        // get "file://c:/WINDOWS/clock.avi", which is specifically called out as invalid
        // on wikipedia, because the "c:" part now looks like a hostname in the
        // "file://hostname/path" format. The egui code does indeed attempt to use this
        // as a UNC network path, and this then fails with OS error 53.
        //
        // On windows we also replace the "\" file separators with "/" - this doesn't seem
        // to be necessary since the path is accepted even with "\", but we might as well be
        // slightly more URI spec. compliant.
        //
        // wikipedia ref: https://en.wikipedia.org/wiki/File_URI_scheme
        #[cfg(target_os = "windows")]
        let uri = format!(
            "tileset://{}//file:///{}",
            mode_json,
            path.as_str().replace("\\", "/")
        );

        #[cfg(not(target_os = "windows"))]
        let uri = format!("tileset://{}//file://{}", mode_json, path.as_str());

        uri
    }

    pub fn path_for_tileset(&self, tileset: &Tileset) -> Option<Utf8PathBuf> {
        match &self.source {
            TextureSource::Builtin => None,
            TextureSource::File { base_dir } => {
                Some(self.path_for_tileset_from_base_dir(base_dir, tileset))
            }
        }
    }

    pub fn texture_for_tileset(
        &self,
        ctx: &Context,
        tileset: &Tileset,
    ) -> Result<TexturePoll, Box<TilesetError>> {
        match &self.source {
            TextureSource::Builtin => MOUNTAIN_TILES
                .load(ctx, TextureOptions::NEAREST, Self::SIZE_HINT)
                .map_err(|load_error| {
                    Box::new(TilesetError {
                        source: self.source.clone(),
                        tileset_path: tileset.path.clone(),
                        resolved_path: tileset.path.clone(),
                        path_status: PathStatus::ExistsAsFile,
                        load_error,
                    })
                }),
            TextureSource::File { base_dir } => {
                let path = self.path_for_tileset_from_base_dir(base_dir, tileset);
                let uri = Self::uri_for_path(&path, &tileset.mode);

                ctx.try_load_texture(&uri, TextureOptions::NEAREST, Self::SIZE_HINT)
                    .map_err(|load_error| {
                        let path_status = Self::path_status(&path);
                        Box::new(TilesetError {
                            source: self.source.clone(),
                            tileset_path: tileset.path.clone(),
                            resolved_path: path.clone(),
                            path_status,
                            load_error,
                        })
                    })
            }
        }
    }

    /// Update base dir using a file path - the base dir is taken to be the parent of the
    /// specified path (or the path itself if it has no parent).
    pub fn update_base_dir_from_file_path(&mut self, path: Option<Utf8PathBuf>) {
        match self.source {
            TextureSource::Builtin => {}
            TextureSource::File { base_dir: _ } => {
                let base_dir = path.map(|mut path| {
                    path.pop();
                    path
                });
                self.source = TextureSource::File { base_dir };
            }
        }
    }

    pub fn builtin_tileset_textures() -> TilesetTextures {
        TilesetTextures {
            source: TextureSource::Builtin,
        }
    }

    pub fn error_texture(
        &self,
        ctx: &Context,
        error_texture: ErrorTexture,
    ) -> Result<TexturePoll, LoadError> {
        let image_source = match error_texture {
            ErrorTexture::MissingPalette => MISSING_PALETTE,
            ErrorTexture::MissingImage => MISSING_IMAGE,
            ErrorTexture::MissingTileset => MISSING_TILESET,
        };
        image_source.load(ctx, TextureOptions::NEAREST, Self::SIZE_HINT)
    }

    pub fn refresh_tileset(&self, ctx: &Context, tileset: &Tileset) {
        match &self.source {
            TextureSource::Builtin => {}
            TextureSource::File { base_dir } => {
                let path = self.path_for_tileset_from_base_dir(base_dir, tileset);
                let uri = Self::uri_for_path(&path, &tileset.mode);
                ctx.forget_image(&uri);
            }
        }
    }
}
