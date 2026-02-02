use std::path::PathBuf;

use camino::Utf8PathBuf;
use eyre::Context;

use crate::data::file_format;

pub const MOUNTAIN_TILES_MAP_NAME: &str = file_format::NAME;
pub const MOUNTAIN_TILES_MAP_EXTENSION: &str = file_format::EXTENSION;

#[cfg(not(target_os = "windows"))]
pub const PNG_NAME: &str = "PNG Image (.png)";

#[cfg(target_os = "windows")]
pub const PNG_NAME: &str = "PNG Image";

pub const PNG_EXTENSION: &str = "png";

#[cfg(not(target_os = "windows"))]
pub const TMX_NAME: &str = "Tiled Map XML (.tmx)";

#[cfg(target_os = "windows")]
pub const TMX_NAME: &str = "Tiled Map XML";

pub const TMX_EXTENSION: &str = "tmx";
pub const TSX_EXTENSION: &str = "tsx";

#[cfg(not(target_os = "windows"))]
pub const RS_NAME: &str = "Rust codegen (.rs)";

#[cfg(target_os = "windows")]
pub const RS_NAME: &str = "Rust codegen";

pub const RS_EXTENSION: &str = "rs";

pub fn optional_pathbuf_to_utf8(pathbuf: Option<PathBuf>) -> eyre::Result<Option<Utf8PathBuf>> {
    match pathbuf {
        Some(pathbuf) => {
            let utf8_pathbuf = Utf8PathBuf::try_from(pathbuf).wrap_err_with(|| {
                "Selected file path contains non-UTF8 characters, and so cannot be used."
            })?;
            Ok(Some(utf8_pathbuf))
        }
        None => Ok(None),
    }
}

/// Pick a file to open, with no filter
/// Returns an error if a file is selected, and the file path can't be converted to a Utf8 path.
/// Otherwise returns the selected file path as utf8, or `None` if no file is selected.
pub fn pick_file() -> eyre::Result<Option<Utf8PathBuf>> {
    optional_pathbuf_to_utf8(rfd::FileDialog::new().pick_file())
}

/// Pick a file to open, with a single filter
/// Returns an error if a file is selected, and the file path can't be converted to a Utf8 path.
/// Otherwise returns the selected file path as utf8, or `None` if no file is selected.
pub fn pick_file_with_extension(name: &str, extension: &str) -> eyre::Result<Option<Utf8PathBuf>> {
    let extensions = [extension];
    optional_pathbuf_to_utf8(
        rfd::FileDialog::new()
            .add_filter(name, &extensions)
            .pick_file(),
    )
}

/// Pick a mountain tiles map file to open
/// Returns an error if a file is selected, and the file path can't be converted to a Utf8 path.
/// Otherwise returns the selected file path as utf8, or `None` if no file is selected.
pub fn pick_mnp_file() -> eyre::Result<Option<Utf8PathBuf>> {
    pick_file_with_extension(MOUNTAIN_TILES_MAP_NAME, MOUNTAIN_TILES_MAP_EXTENSION)
}

/// Pick a file to save, with single filter and optional default path for file
/// Returns an error if a file is selected, and the file path can't be converted to a Utf8 path.
/// Otherwise returns the selected file path as utf8, or `None` if no file is selected.
pub fn save_file_with_extension_and_default(
    name: &str,
    extension: &str,
    default_path: &Option<Utf8PathBuf>,
) -> eyre::Result<Option<Utf8PathBuf>> {
    let extensions = [extension];
    let mut dialog = rfd::FileDialog::new().add_filter(name, &extensions);
    if let Some(path) = default_path {
        // Use parent of file path for directory
        let mut directory = path.clone();
        directory.pop();
        dialog = dialog.set_directory(directory);

        if let Some(file_name) = path.file_name() {
            dialog = dialog.set_file_name(file_name);
        }
    }

    optional_pathbuf_to_utf8(dialog.save_file())
}

pub fn save_mnp_file(default_path: &Option<Utf8PathBuf>) -> eyre::Result<Option<Utf8PathBuf>> {
    save_file_with_extension_and_default(
        MOUNTAIN_TILES_MAP_NAME,
        MOUNTAIN_TILES_MAP_EXTENSION,
        default_path,
    )
}

pub fn save_png_file(default_path: &Option<Utf8PathBuf>) -> eyre::Result<Option<Utf8PathBuf>> {
    save_file_with_extension_and_default(PNG_NAME, PNG_EXTENSION, default_path)
}

pub fn save_tmx_file(default_path: &Option<Utf8PathBuf>) -> eyre::Result<Option<Utf8PathBuf>> {
    save_file_with_extension_and_default(TMX_NAME, TMX_EXTENSION, default_path)
}

pub fn save_rs_file(default_path: &Option<Utf8PathBuf>) -> eyre::Result<Option<Utf8PathBuf>> {
    save_file_with_extension_and_default(RS_NAME, RS_EXTENSION, default_path)
}
