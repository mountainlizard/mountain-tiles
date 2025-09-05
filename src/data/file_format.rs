use std::{fs::File, io::BufReader};

use camino::Utf8PathBuf;
use eyre::eyre;

use crate::data::state::State;

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub enum FileFormat {
    #[serde(rename = "mountainlizard.com/mountain-tiles/v0")]
    MountainLizardComMountainTilesV0,
}

impl FileFormat {
    pub const CURRENT: FileFormat = FileFormat::MountainLizardComMountainTilesV0;
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
/// containing the value [`FileFormat::CURRENT`].
/// When more formats (either entirely different formats, or
/// new versions) are implemented, this can be made to accept
/// whichever ones are currently supported, and will return the
/// recognised [`FileFormat`].
/// Note that this is intended to be quicker than trying to open
/// the entire file as a given format, and possibly failing, since
/// it only tries to read a single field, `format`, as a string.
pub fn confirm_format(path: Utf8PathBuf) -> eyre::Result<FileFormat> {
    let file = File::open(path.clone())?;
    let buf_reader = BufReader::new(file);
    let file_contents: MinimalFileContents = serde_json::from_reader(buf_reader)?;
    let expected_format = serde_json::to_string(&FileFormat::CURRENT)?;
    if expected_format == format!("\"{}\"", file_contents.format) {
        Ok(FileFormat::CURRENT)
    } else if file_contents
        .format
        .starts_with("mountainlizard.com/mountain-tiles/")
    {
        Err(eyre!("Unsupported file version - please update software"))
    } else {
        Err(eyre!("Unsupported file format"))
    }
}
