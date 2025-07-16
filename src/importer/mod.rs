use std::path::PathBuf;

use iroh_blobs::
    api::
        Store
    
;
use walkdir::WalkDir;

// Stolen from sendme and stripped.

/// Import from a file or directory into the database.
///
/// The returned tag always refers to a collection. If the input is a file, this
/// is a collection with a single blob, named like the file.
///
/// If the input is a directory, the collection contains all the files in the
/// directory.
pub async fn import(path: PathBuf, _db: &Store) -> anyhow::Result<()> {
    let path = path.canonicalize()?;
    anyhow::ensure!(path.exists(), "path {} does not exist", path.display());
    // let root = path.parent().context("context get parent")?;
    // walkdir also works for files, so we don't need to special case them
    let files = WalkDir::new(path.clone()).into_iter();
    for item in files{ 
        if let Ok(dir_entry) = item {
            if dir_entry.file_type().is_dir() {
                println!("{}",dir_entry.path().display());
            }
            println!("\t{}",dir_entry.path().display());
        }
    }
    Ok(())
}