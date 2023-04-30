use integer_encoding::VarIntReader;
use ipld::{Cid, Ipld};

use crate::{error::CarSplitterError, header::CarHeader, section::Section};

use std::{
    collections::HashMap,
    io::{Read, Seek},
};

pub(crate) struct CarReader<R> {
    inner: R,
    sections: HashMap<Cid, Section>,
    header: CarHeader,
}

impl<R> CarReader<R>
where
    R: Read + Seek,
{
    pub(crate) fn new(mut inner: R) -> Result<Self, CarSplitterError> {
        let header = CarHeader::read_header(&mut inner)?;
        let mut sections = HashMap::new();
        while let Some(section) = read_section(&mut inner)? {
            sections.insert(section.cid(), section);
        }
        Ok(Self {
            inner,
            header,
            sections,
        })
    }

    #[inline(always)]
    pub fn header(&self) -> &CarHeader {
        &self.header
    }

    #[inline]
    pub fn read_section_data(&mut self, cid: &Cid) -> Result<Vec<u8>, CarSplitterError> {
        let s = self
            .sections
            .get(cid)
            .ok_or(CarSplitterError::InvalidSection("CID not exist".into()))?;
        s.read_data(&mut self.inner)
    }

    #[inline]
    pub fn ipld(&mut self, cid: &Cid) -> Result<Ipld, CarSplitterError> {
        let s = self
            .sections
            .get_mut(cid)
            .ok_or(CarSplitterError::NotFound("CID not exist".into()))?;
        s.ipld(&mut self.inner)
    }
}

const MAX_ALLOWED_SECTION_SIZE: usize = 32 << 20;

pub fn read_block<R>(mut reader: R) -> Result<Option<Vec<u8>>, CarSplitterError>
where
    R: std::io::Read,
{
    let l: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarSplitterError::Io(e));
        }
    };
    if l > MAX_ALLOWED_SECTION_SIZE {
        return Err(CarSplitterError::TooLargeSection(l));
    }
    let mut data = vec![0u8; l];
    reader.read_exact(&mut data[..])?;
    Ok(Some(data))
}

fn read_section<R>(mut reader: R) -> Result<Option<Section>, CarSplitterError>
where
    R: std::io::Read + std::io::Seek,
{
    let len: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarSplitterError::Io(e));
        }
    };
    let start = reader.stream_position()?;
    if len > MAX_ALLOWED_SECTION_SIZE {
        return Err(CarSplitterError::TooLargeSection(len));
    }
    let cid = Cid::read_bytes(&mut reader).map_err(|e| CarSplitterError::Parsing(e.to_string()))?;
    let pos = reader.stream_position()?;
    let l = len - ((pos - start) as usize);
    reader.seek(std::io::SeekFrom::Current(l as _))?;
    Ok(Some(Section::new(cid, pos, l)))
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        todo!("test");
    }
}
