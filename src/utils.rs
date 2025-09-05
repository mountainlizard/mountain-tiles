use camino::{Utf8Path, Utf8PathBuf};
use eyre::eyre;

pub fn tmx_parent_dir(tmx_path: &Utf8PathBuf) -> eyre::Result<&Utf8Path> {
    tmx_path
        .parent()
        .ok_or_else(|| eyre!("Tiled .tmx file path has no parent, cannot import"))
}

pub fn tsx_parent_dir(tsx_path: &Utf8PathBuf) -> eyre::Result<&Utf8Path> {
    tsx_path
        .parent()
        .ok_or_else(|| eyre!("Tiled .tsx file path has no parent, cannot import"))
}

/// Produce a new [`Utf8PathBuf`] based on an existing one, but with:
/// 1. A new filename stem, this is the old filename stem then a "-", then the suffix.
///    If the path doesn't have a filename stem (because it doesn't have a filename)
///    then we use the `default_stem` instead, and the path will have this new filename
///    added.
/// 2. Any existing extension replaced by `extension`
pub fn path_with_suffix_and_extension(
    path: &Utf8PathBuf,
    default_stem: &str,
    suffix: &str,
    extension: &str,
) -> Utf8PathBuf {
    let mut path = path.clone();
    path.set_file_name(format!(
        "{}-{}",
        path.file_stem().unwrap_or(default_stem),
        suffix
    ));
    path.set_extension(extension);
    path
}
