use byteorder::{ReadBytesExt, WriteBytesExt};

pub struct Mpak {
    entries: indexmap::IndexMap<u32, Vec<u8>>,
}

struct MapHeader {
    count: u32,
    rom_addr_min: u32,
    rom_addr_max: u32,
}

struct MapEntry {
    rom_addr: u32,
    mpak_offset: u32,
    mpak_size: u32,
}

impl MapHeader {
    pub fn read_from(mut r: impl std::io::Read) -> Result<Self, std::io::Error> {
        Ok(MapHeader {
            count: r.read_u32::<byteorder::LittleEndian>()?,
            rom_addr_min: r.read_u32::<byteorder::LittleEndian>()?,
            rom_addr_max: r.read_u32::<byteorder::LittleEndian>()?,
        })
    }

    pub fn write_into(&self, mut w: impl std::io::Write) -> Result<(), std::io::Error> {
        w.write_u32::<byteorder::LittleEndian>(self.count)?;
        w.write_u32::<byteorder::LittleEndian>(self.rom_addr_min)?;
        w.write_u32::<byteorder::LittleEndian>(self.rom_addr_max)?;
        Ok(())
    }
}

impl MapEntry {
    pub fn read_from(mut r: impl std::io::Read) -> Result<Self, std::io::Error> {
        Ok(MapEntry {
            rom_addr: r.read_u32::<byteorder::LittleEndian>()?,
            mpak_offset: r.read_u32::<byteorder::LittleEndian>()?,
            mpak_size: r.read_u32::<byteorder::LittleEndian>()?,
        })
    }

    pub fn write_into(&self, mut w: impl std::io::Write) -> Result<(), std::io::Error> {
        w.write_u32::<byteorder::LittleEndian>(self.rom_addr)?;
        w.write_u32::<byteorder::LittleEndian>(self.mpak_offset)?;
        w.write_u32::<byteorder::LittleEndian>(self.mpak_size)?;
        Ok(())
    }
}

impl Mpak {
    pub fn read_from(
        mut map_reader: impl std::io::Read,
        mut mpak_reader: impl std::io::Read + std::io::Seek,
    ) -> Result<Self, std::io::Error> {
        // Read the entire mpak into memory: who cares, it's not very expensive.
        let header = MapHeader::read_from(&mut map_reader)?;
        let mut entries = indexmap::IndexMap::new();
        for _ in 0..header.count {
            let entry = MapEntry::read_from(&mut map_reader)?;
            let mut buf = vec![0; entry.mpak_size as usize];
            mpak_reader.seek(std::io::SeekFrom::Start(entry.mpak_offset as u64))?;
            mpak_reader.read_exact(buf.as_mut_slice())?;
            entries.insert(entry.rom_addr, buf);
        }
        Ok(Self { entries })
    }

    pub fn insert(&mut self, rom_addr: u32, contents: Vec<u8>) -> Option<Vec<u8>> {
        self.entries.insert(rom_addr, contents)
    }

    pub fn get<'a>(&'a self, rom_addr: u32) -> Option<&'a [u8]> {
        self.entries.get(&rom_addr).map(|v| &v[..])
    }

    pub fn write_into(
        &self,
        mut map_writer: impl std::io::Write,
        mut mpak_writer: impl std::io::Write,
    ) -> Result<(), std::io::Error> {
        MapHeader {
            count: self.entries.len() as u32,
            rom_addr_min: self.entries.keys().min().map(|v| *v).unwrap_or(0),
            rom_addr_max: self.entries.keys().max().map(|v| *v).unwrap_or(0),
        }
        .write_into(&mut map_writer)?;

        let mut mpak_offset = 0;
        for (rom_addr, entry) in self.entries.iter() {
            MapEntry {
                rom_addr: *rom_addr,
                mpak_offset: mpak_offset as u32,
                mpak_size: entry.len() as u32,
            }
            .write_into(&mut map_writer)?;
            mpak_writer.write_all(entry)?;
            mpak_offset += entry.len();
        }

        Ok(())
    }
}
