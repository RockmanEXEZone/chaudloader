use byteorder::{ReadBytesExt, WriteBytesExt};

pub fn unpack(mut r: impl std::io::Read) -> Result<Vec<Vec<u8>>, std::io::Error> {
    // Read offsets table.
    let first_offset = r.read_u16::<byteorder::LittleEndian>()? as usize;
    let n = first_offset / 2;

    let mut offsets = Vec::with_capacity(n);
    offsets.push(first_offset);
    for _ in 1..n {
        offsets.push(r.read_u16::<byteorder::LittleEndian>()? as usize);
    }

    // Read entries.
    let mut entries = Vec::with_capacity(n);
    for len in std::iter::zip(offsets.iter(), offsets[1..].iter())
        .map(|(x, y)| Some(y - x))
        .chain(std::iter::once(None))
    {
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
    let mut offset = (entries.len() * 2) as usize;

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
            vec![b"hello".to_vec(), b"world".to_vec(),]
        );
    }
}