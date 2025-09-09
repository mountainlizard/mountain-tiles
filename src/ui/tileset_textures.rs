use std::fmt::Display;

use crate::data::tilesets::{Tileset, TilesetMode};
use camino::Utf8PathBuf;
use egui::{
    emath::OrderedFloat,
    load::{LoadError, TexturePoll},
    Context, ImageSource, SizeHint, TextureOptions,
};

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
                    format!("Image has absolute path '{}', is this correct for your system? You might want to use a relative path.", self.tileset_path)
                } else {
                    match base_dir {
                        None => format!("Image has relative path '{}', but project has not been saved, try saving.", self.tileset_path),
                        Some(base_dir) => format!("Image is at relative path '{}', project saved at '{}'.", self.tileset_path, base_dir),
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
        if let Some(ref base_dir) = base_dir {
            let mut path = base_dir.clone();
            path.push(tileset.path.clone());
            path
        } else {
            tileset.path.clone()
        }
    }

    pub fn uri_for_path(path: &Utf8PathBuf, mode: &TilesetMode) -> String {
        let mode_json = serde_json::to_string(mode).unwrap_or("\"Direct\"".to_string());
        format!("tileset://{}//file://{}", mode_json, path)
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
