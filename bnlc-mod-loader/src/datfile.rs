#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("persist: {0}")]
    Persist(#[from] tempfile::PersistError),

    #[error("cannot replace directory")]
    CannotReplaceDirectory,
}

pub struct Repacker<R> {
    zr: zip::ZipArchive<R>,
    replacements: std::collections::HashMap<String, Box<dyn std::io::Read>>,
}

impl<R> Repacker<R>
where
    R: std::io::Read + std::io::Seek,
{
    pub fn new(reader: R) -> Result<Self, Error> {
        Ok(Self {
            zr: zip::ZipArchive::new(reader)?,
            replacements: std::collections::HashMap::new(),
        })
    }

    pub fn replace(&mut self, path: &str, r: impl std::io::Read + 'static) -> Result<(), Error> {
        if self.zr.by_name(path)?.is_dir() {
            return Err(Error::CannotReplaceDirectory);
        }
        self.replacements.insert(path.to_owned(), Box::new(r));
        Ok(())
    }

    pub fn finish(mut self, writer: impl std::io::Write + std::io::Seek) -> Result<(), Error> {
        let mut zw = zip::ZipWriter::new(writer);
        for i in 0..self.zr.len() {
            let entry = self.zr.by_index_raw(i)?;
            if let Some(replacement) = self.replacements.get_mut(entry.name()) {
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
}
