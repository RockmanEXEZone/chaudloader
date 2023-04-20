#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

fn repack_into(
    reader: impl std::io::Read + std::io::Seek,
    mut replacements: std::collections::HashMap<String, Box<dyn std::io::Read>>,
    writer: impl std::io::Write + std::io::Seek,
) -> Result<(), Error> {
    let mut zr = zip::ZipArchive::new(reader)?;
    let mut zw = zip::ZipWriter::new(writer);
    for i in 0..zr.len() {
        let entry = zr.by_index_raw(i)?;
        if let Some(replacement) = replacements.get_mut(entry.name()) {
            log::info!("replacing {}", entry.name());
            zw.start_file(
                entry.name(),
                zip::write::FileOptions::default().compression_method(entry.compression()),
            )?;
            std::io::copy(replacement, &mut zw)?;
        } else {
            zw.raw_copy_file(entry)?;
        }
    }
    Ok(())
}

// Remember all of our repacked files so we can run destructors on exit.
//
// TODO: Run destructors on exit.
static REPACKED_FILES: std::sync::LazyLock<std::sync::Mutex<Vec<tempfile::NamedTempFile>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(vec![]));

/// Repacks a .dat (zip) archive.
///
/// Files may be replaced via the replacements map, which contains a single-use reader to read from for each entry to replace the asset's contents with.
pub fn repack(
    path: &std::path::Path,
    replacements: std::collections::HashMap<String, Box<dyn std::io::Read>>,
) -> Result<std::path::PathBuf, Error> {
    let mut src_f = std::fs::File::open(path)?;
    let mut dest_f = tempfile::NamedTempFile::new()?;
    repack_into(&mut src_f, replacements, &mut dest_f)?;
    let path = dest_f.path().to_owned();
    let mut repacked_files = REPACKED_FILES.lock().unwrap();
    repacked_files.push(dest_f);
    Ok(path)
}
