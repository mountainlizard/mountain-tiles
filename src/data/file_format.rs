use std::{fmt::Display, fs::File, io::BufReader};

use camino::Utf8PathBuf;
use eyre::eyre;
use serde_json::Value;

use crate::data::state::State;

/// The mime type for mountain tiles data files
pub const MIME_TYPE: &str = "application/x-mountain-tiles";

#[cfg(not(target_os = "windows"))]
/// The display name for mountain tiles data files
/// On non-Windows OSs the dialog (usually?) doesn't display
/// the extension directly alongside the filter name, so we
/// add it for clarity
pub const NAME: &str = "MountainTiles Map (.mnp)";

#[cfg(target_os = "windows")]
/// The display name for mountain tiles data files
/// On Windows the dialog displays the extension alongside
/// the filter name, so we don't have to
pub const NAME: &str = "MountainTiles Map";

/// The filename extension for mountain tiles data files
pub const EXTENSION: &str = "mnp";

/// The different file formats supported in a mountain tiles data file.
/// While all files should have the same [`MIME_TYPE`] and [`EXTENSION`],
/// the contents may differ in format. All contents match the
/// [`MinimalFileContents`] struct by having at least the `format` field,
/// and this must be a value from this [`FileFormat`] enum, which is used
/// to identify the rest of the contents to allow decoding as the appropriate
/// struct.
/// This allows for both incompatible changes to the version of the file, and
/// possibly for files with different contents in future. Currently only
/// [`FileContents`] is supported.
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum FileFormat {
    /// A mountain tiles file containing a [`FileContents`] struct
    /// encoded in json. This in turn contains this [`FileFormat`]
    /// itself as a `format` field, and a [`State`] containing
    /// resources, maps etc.
    /// This uses the original data model for the [`State`].
    #[serde(rename = "mountainlizard.com/mountain-tiles/v0")]
    MountainLizardComMountainTilesV0,

    /// Identical to [`FileFormat::MountainLizardComMountainTilesV0`], except
    /// for the following change to the data in [`State`]:
    ///
    /// 1. [`crate::data::tiles::tile_color::TileColor`] has a new variant
    ///    for foreground + background color
    ///
    /// All [`FileFormat::MountainLizardComMountainTilesV0`] files can be read as
    /// [`FileFormat::MountainLizardComMountainTilesV1`], but any
    /// [`FileFormat::MountainLizardComMountainTilesV1`] files using the new variant
    /// can't be read in older software. Incrementing the file format prevents older
    /// software trying and failing to read the new variant.
    #[serde(rename = "mountainlizard.com/mountain-tiles/v1")]
    MountainLizardComMountainTilesV1,
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::MountainLizardComMountainTilesV0 => {
                write!(f, "mountainlizard.com mountain-tiles file format v0")
            }
            FileFormat::MountainLizardComMountainTilesV1 => {
                write!(f, "mountainlizard.com mountain-tiles file format v1")
            }
        }
    }
}

impl FileFormat {
    /// The current main file format, bumped when we have a new version.
    pub const CURRENT: FileFormat = FileFormat::MountainLizardComMountainTilesV1;
}

/// Contents of file - we only need the data version in files, so we
/// make a temporary structure to save to json
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct FileContents {
    pub format: FileFormat,
    pub state: State,
}

/// Minimal contents of file - we can deserialise to this to check the
/// format, without needing to parse anything else, and with support for
/// unknown format fields
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct MinimalFileContents {
    pub format: String,
}

/// Confirm that a file is in a supported format, which is
/// any JSON file containing an object with a field `format`
/// containing the value [`FileFormat::CURRENT`], or an acceptable
/// older [`FileFormat`].
/// Note that this is intended to be quicker than trying to open
/// the entire file as a given format, and possibly failing, since
/// it only tries to read a single field, `format`, as a string.
pub fn confirm_format(path: Utf8PathBuf) -> eyre::Result<FileFormat> {
    let file = File::open(path.clone())?;
    let buf_reader = BufReader::new(file);
    let file_contents: MinimalFileContents = serde_json::from_reader(buf_reader)?;
    let format = file_contents.format;

    // First check basic format id - if it doesn't start with
    // our URL, then it can't be from a valid mountain-tiles file,
    // even a future version, and there's no point recommending upgrading
    if !format.starts_with("mountainlizard.com/mountain-tiles/") {
        Err(eyre!("Unsupported file format"))

    // Now try to parse the format id as a known [`FileFormat`]
    } else {
        // Encode the format as JSON (e.g. adding quotes), so we can attempt
        // to decode it as a JSON-encoded [`FileFormat`]. We could use strum but
        // might as well reuse JSON parsing since we need it anyway.
        let format_json = Value::String(format).to_string();
        serde_json::from_str(&format_json)
            .map_err(|_e| eyre!("Unsupported file version - please update software"))
    }
}
