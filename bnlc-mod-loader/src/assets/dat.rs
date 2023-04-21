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

    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}

pub struct Reader<R> {
    zr: zip::ZipArchive<R>,
}

impl<R> Reader<R>
where
    R: std::io::Read + std::io::Seek,
{
    pub fn new(reader: R) -> Result<Self, zip::result::ZipError> {
        Ok(Self {
            zr: zip::ZipArchive::new(reader)?,
        })
    }

    pub fn get<'a>(
        &'a mut self,
        path: &str,
    ) -> Result<zip::read::ZipFile<'a>, zip::result::ZipError> {
        Ok(self.zr.by_name(path)?)
    }
}

pub struct Repacker<R> {
    zr: zip::ZipArchive<R>,
    replacements: std::collections::HashMap<
        String,
        Box<dyn Fn(&mut dyn std::io::Write) -> Result<(), anyhow::Error>>,
    >,
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

    pub fn replace(
        &mut self,
        path: &str,
        replacer: impl Fn(&mut dyn std::io::Write) -> Result<(), anyhow::Error> + 'static,
    ) -> Result<(), Error> {
        if self.zr.by_name(path)?.is_dir() {
            return Err(Error::CannotReplaceDirectory);
        }
        self.replacements
            .insert(path.to_owned(), Box::new(replacer));
        Ok(())
    }

    pub fn pack_into(mut self, writer: impl std::io::Write + std::io::Seek) -> Result<(), Error> {
        let mut zw = zip::ZipWriter::new(writer);
        for i in 0..self.zr.len() {
            let entry = self.zr.by_index_raw(i)?;
            if let Some(replacer) = self.replacements.get(entry.name()) {
                log::info!("replacing {}", entry.name());
                zw.start_file(
                    entry.name(),
                    zip::write::FileOptions::default().compression_method(entry.compression()),
                )?;
                replacer(&mut zw)?;
            } else {
                zw.raw_copy_file(entry)?;
            }
        }
        Ok(())
    }
}
