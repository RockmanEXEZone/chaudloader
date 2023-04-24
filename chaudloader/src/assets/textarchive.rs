use byteorder::{ReadBytesExt, WriteBytesExt};

pub fn unpack(mut r: impl std::io::Read) -> Result<Vec<Vec<u8>>, std::io::Error> {
    // Read offsets table.
    let first_offset = r.read_u16::<byteorder::LittleEndian>()? as usize;
    let n = first_offset / std::mem::size_of::<u16>();

    let mut offsets = Vec::with_capacity(n);
    offsets.push(first_offset);
    for _ in 1..n {
        offsets.push(r.read_u16::<byteorder::LittleEndian>()? as usize);
    }

    // Read entries.
    let mut entries = Vec::with_capacity(n);
    for (i, len) in std::iter::zip(offsets.iter(), offsets[1..].iter())
        .map(|(x, y)| y.checked_sub(*x).map(|v| Some(v)))
        .chain(std::iter::once(Some(None)))
        .enumerate()
    {
        let len = if let Some(len) = len {
            len
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("offset {} went backwards", i),
            ));
        };

        entries.push(if let Some(len) = len {
            let mut buf = vec![0; len];
            r.read_exact(&mut buf)?;
            buf
        } else {
            let mut buf = vec![];
            r.read_to_end(&mut buf)?;
            buf
        });
    }

    Ok(entries)
}

pub fn pack(entries: &[&[u8]], mut w: impl std::io::Write) -> Result<(), std::io::Error> {
    let mut offset = (entries.len() * std::mem::size_of::<u16>()) as usize;

    // Write offsets table.
    for entry in entries.iter() {
        w.write_u16::<byteorder::LittleEndian>(offset as u16)?;
        offset += entry.len();
    }

    // Write entries.
    for entry in entries.iter() {
        w.write_all(entry)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_roundtrip() {
        let mut buf = vec![];
        pack(&[b"hello", b"world"], &mut buf).unwrap();
        assert_eq!(buf, b"\x04\x00\x09\x00helloworld");
        assert_eq!(
            unpack(std::io::Cursor::new(&buf)).unwrap(),
            vec![b"hello".to_vec(), b"world".to_vec()]
        );
    }

    #[test]
    fn test_pack_unpack_bad_offsets() {
        let buf = b"\x04\x03uhoh";
        assert_eq!(
            unpack(std::io::Cursor::new(&buf)).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
    }
}
