use std::io::{Read, Write};

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

pub struct Reader {
    zr: zip::ZipArchive<Box<dyn super::ReadSeek + Send>>,
}

impl Reader {
    pub fn new(
        reader: impl super::ReadSeek + Send + 'static,
    ) -> Result<Self, zip::result::ZipError> {
        Ok(Self {
            zr: zip::ZipArchive::new(Box::new(reader) as Box<dyn super::ReadSeek + Send>)?,
        })
    }

    pub fn get<'a>(
        &'a mut self,
        path: &str,
    ) -> Result<zip::read::ZipFile<'a>, zip::result::ZipError> {
        Ok(self.zr.by_name(path)?)
    }
}

pub struct Overlay {
    base: Reader,
    overlaid_files: std::collections::HashMap<String, Vec<u8>>,
}

impl Overlay {
    pub fn new(base: Reader) -> Self {
        Self {
            base,
            overlaid_files: std::collections::HashMap::new(),
        }
    }

    pub fn read<'a>(&'a mut self, path: &str) -> Result<std::borrow::Cow<'a, [u8]>, Error> {
        if let Some(contents) = self.overlaid_files.get(path) {
            return Ok(std::borrow::Cow::Borrowed(&contents));
        }
        let mut zf = self.base.get(path)?;
        let mut buf = vec![];
        zf.read_to_end(&mut buf)?;
        Ok(std::borrow::Cow::Owned(buf))
    }

    pub fn write<'a>(&'a mut self, path: &str, contents: Vec<u8>) -> Result<(), Error> {
        if self.base.get(path)?.is_dir() {
            return Err(Error::CannotReplaceDirectory);
        }
        self.overlaid_files.insert(path.to_string(), contents);
        Ok(())
    }

    pub fn has_overlaid_files(&self) -> bool {
        !self.overlaid_files.is_empty()
    }

    pub fn pack_into(&mut self, writer: impl std::io::Write + std::io::Seek) -> Result<(), Error> {
        let mut zw = zip::ZipWriter::new(writer);
        for i in 0..self.base.zr.len() {
            let entry = self.base.zr.by_index_raw(i)?;
            if let Some(contents) = self.overlaid_files.get(entry.name()) {
                log::info!("replacing {}", entry.name());
                zw.start_file(
                    entry.name(),
                    zip::write::FileOptions::default().compression_method(entry.compression()),
                )?;
                zw.write_all(&contents)?;
            } else {
                zw.raw_copy_file(entry)?;
            }
        }
        Ok(())
    }
}
