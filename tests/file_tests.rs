use std::path;

use assert_fs::{fixture::PathChild, TempDir};
use camino::Utf8PathBuf;
use eyre::{eyre, Result};
use mountain_tiles::data::state::State;

trait TempDirUtf8 {
    fn child_utf8_path_buf<P>(&self, path: P) -> Result<Utf8PathBuf>
    where
        P: AsRef<path::Path>;
}

impl TempDirUtf8 for TempDir {
    fn child_utf8_path_buf<P>(&self, path: P) -> Result<Utf8PathBuf>
    where
        P: AsRef<path::Path>,
    {
        let path_buf = self.child(path).to_path_buf();
        let utf8 = Utf8PathBuf::from_path_buf(path_buf)
            .map_err(|_| eyre!("Can't convert temp file to utf8"))?;
        Ok(utf8)
    }
}

#[test]
fn test_open_and_save() -> Result<(), eyre::Error> {
    let temp = assert_fs::TempDir::new()?;

    let mut map = State::from_path("test-data/instructions-v0.mnp".into())?;

    let output = temp.child_utf8_path_buf("instructions-v0-out.mnp")?;
    map.save_to_path(output)?;

    Ok(())
}
