use std::io::Read;

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
    zr: zip::ZipArchive<Box<dyn super::ReadSeek>>,
}

impl Reader {
    pub fn new(reader: impl super::ReadSeek + 'static) -> Result<Self, zip::result::ZipError> {
        Ok(Self {
            zr: zip::ZipArchive::new(Box::new(reader) as Box<dyn super::ReadSeek>)?,
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

    pub fn write<'a>(&'a mut self, path: &str, contents: Vec<u8>) {
        self.overlaid_files.insert(path.to_string(), contents);
    }

    pub fn into_overlaid_files(self) -> std::collections::HashMap<String, Vec<u8>> {
        self.overlaid_files
    }
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
