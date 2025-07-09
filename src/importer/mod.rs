use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use iroh_blobs::{
    api::{
        Store, TempTag,
        blobs::AddProgressItem,
    },
    format::collection::Collection,
};
use n0_future::StreamExt;
use walkdir::WalkDir;
use chrono::Local;

// Stolen from sendme and stripped.

/// This function converts an already canonicalized path to a string.
///
/// If `must_be_relative` is true, the function will fail if any component of the path is
/// `Component::RootDir`
///
/// This function will also fail if the path is non canonical, i.e. contains
/// `..` or `.`, or if the path components contain any windows or unix path
/// separators.
pub fn canonicalized_path_to_string(
    path: impl AsRef<Path>,
    must_be_relative: bool,
) -> anyhow::Result<String> {
    let mut path_str = String::new();
    let parts = path
        .as_ref()
        .components()
        .filter_map(|c| match c {
            Component::Normal(x) => {
                let c = match x.to_str() {
                    Some(c) => c,
                    None => return Some(Err(anyhow::anyhow!("invalid character in path"))),
                };

                if !c.contains('/') && !c.contains('\\') {
                    Some(Ok(c))
                } else {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                }
            }
            Component::RootDir => {
                if must_be_relative {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                } else {
                    path_str.push('/');
                    None
                }
            }
            _ => Some(Err(anyhow::anyhow!("invalid path component {:?}", c))),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let parts = parts.join("/");
    path_str.push_str(&parts);
    Ok(path_str)
}
/// Import from a file or directory into the database.
///
/// The returned tag always refers to a collection. If the input is a file, this
/// is a collection with a single blob, named like the file.
///
/// If the input is a directory, the collection contains all the files in the
/// directory.
pub async fn import(path: PathBuf, db: &Store) -> anyhow::Result<()> {
    let path = path.canonicalize()?;
    anyhow::ensure!(path.exists(), "path {} does not exist", path.display());
    let root = path.parent().context("context get parent")?;
    // walkdir also works for files, so we don't need to special case them
    let files = WalkDir::new(path.clone()).into_iter();
    // flatten the directory structure into a list of (name, path) pairs.
    // ignore symlinks.
    let data_sources: Vec<(String, PathBuf)> = files
        .map(|entry| {
            let entry = entry?;
            if !entry.file_type().is_file() {
                // Skip symlinks. Directories are handled by WalkDir.
                return Ok(None);
            }
            let path = entry.into_path();
            let relative = path.strip_prefix(root)?;
            let name = canonicalized_path_to_string(relative, true)?;
            anyhow::Ok(Some((name, path)))
        })
        .filter_map(Result::transpose)
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut names_and_tags = Vec::<(String, TempTag)>::new();

    for (_rel_path, full_path) in data_sources {
        //println!("{:?}", full_path);
        let imp = db.add_path(&full_path);
        let mut stream = imp.stream().await;
        let temp_tag = loop {
            let item = stream.next().await.context("no tag")?;
            match item {
                // AddProgressItem::CopyProgress(_) => todo!(),
                // AddProgressItem::Size(_) => todo!(),
                // AddProgressItem::CopyDone => todo!(),
                // AddProgressItem::OutboardProgress(_) => todo!(),
                AddProgressItem::Done(temp_tag) => break temp_tag,
                AddProgressItem::Error(error) => println!("Error : {:?}", error),
                _ => {}
            }
        };
        // db.tags()
            // .set(&full_path.display().to_string(), *temp_tag.hash())
            // .await?;
        names_and_tags.push((full_path.display().to_string(), temp_tag));
    }
    let (collection,_data) = names_and_tags
        .iter()
        .map(|(name, tag)| ((name, *tag.hash()),"dasfasdf"))
        .unzip::<_, _, Collection, Vec<_>>();
    println!("Collection -- {:?}",collection);
    let d  = collection.clone().store(db).await?;
    println!("{:?}",d);
    let dt = Local::now().to_rfc3339().to_owned();
    db.tags().set(format!("col-{}",dt),*d.hash()).await?;
    Ok(())
    // let (collection, tags) = names_and_tags
    //     .into_iter()
    //     .map(|(name, tag, _)| ((name, *tag.hash()), tag))
    //     .unzip::<_, _, Collection, Vec<_>>();
    //let temp_tag = collection.clone().store(db).await?;

    // now that the collection is stored, we can drop the tags
    // data is protected by the collection
    // drop(tags);
}
