#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn repack(
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
