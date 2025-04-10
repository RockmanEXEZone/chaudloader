use std::io::{Read, Write};

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
    fn correct_path(&mut self, path: &str) -> String {
        let filecheck = self.base.get(path);

        match filecheck {
            Ok(_) => path.into(),
            Err(_) => {
                if path.contains("/") {
                    return path.replace("/", "\\");
                }

                path.replace("\\", "/")
            }
        }
    }

    pub fn read<'a>(
        &'a mut self,
        path: &str,
    ) -> Result<std::borrow::Cow<'a, [u8]>, std::io::Error> {
        let correctpath = self.correct_path(path);

        if let Some(contents) = self.overlaid_files.get(correctpath.as_str()) {
            return Ok(std::borrow::Cow::Borrowed(&contents));
        }
        let mut zf = self.base.get(correctpath.as_str())?;
        let mut buf = vec![];
        zf.read_to_end(&mut buf)?;
        Ok(std::borrow::Cow::Owned(buf))
    }

    pub fn write<'a>(&'a mut self, path: &str, contents: Vec<u8>) -> Result<(), std::io::Error> {
        let correctpath = self.correct_path(path);
        if self.base.get(correctpath.as_str())?.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot replace directory",
            ));
        }
        self.overlaid_files.insert(correctpath, contents);
        Ok(())
    }

    pub fn has_overlaid_files(&self) -> bool {
        !self.overlaid_files.is_empty()
    }

    pub fn pack_into(
        &mut self,
        writer: impl std::io::Write + std::io::Seek,
    ) -> Result<(), std::io::Error> {
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

pub fn scan() -> Result<std::collections::HashMap<String, Overlay>, std::io::Error> {
    let mut overlays = std::collections::HashMap::new();
    for entry in std::fs::read_dir("data")? {
        let entry = entry?;
        if entry.path().extension() != Some(&std::ffi::OsStr::new("dat")) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("exe") && file_name != "reader.dat" && file_name != "rkb.dat" {
            continue;
        }

        let src_f = std::fs::File::open(&entry.path())?;
        let reader = Reader::new(src_f)?;

        let overlay = Overlay::new(reader);
        overlays.insert(file_name, overlay);
    }
    Ok(overlays)
}
