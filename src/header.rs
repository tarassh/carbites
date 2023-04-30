use ipld::{prelude::Codec, Cid};
use ipld_cbor::DagCborCodec;
use std::io::{Read, Seek};

use crate::error::CarSplitterError;
use crate::reader::read_block;

#[derive(Debug, Clone, PartialEq, Eq, Default, ipld::DagCbor)]
pub(crate) struct CarHeader {
    #[ipld]
    pub roots: Vec<Cid>,
    #[ipld]
    pub version: u64,
}

impl CarHeader {
    pub fn new(roots: Vec<Cid>) -> Self {
        Self { roots, version: 1 }
    }

    pub fn read_header<R>(r: R) -> Result<CarHeader, CarSplitterError>
    where
        R: Read + Seek,
    {
        let data = match read_block(r) {
            Ok(Some(d)) => d,
            Ok(None) => return Err(CarSplitterError::Parsing("invalid Header".into())),
            Err(e) => return Err(e),
        };
        let header = CarHeader::decode(&data[..])?;
        Ok(header)
    }

    pub fn decode(buf: &[u8]) -> Result<CarHeader, CarSplitterError> {
        let header: CarHeader = DagCborCodec
            .decode(buf)
            .map_err(|e| CarSplitterError::Parsing(e.to_string()))?;
        if header.roots.is_empty() {
            return Err(CarSplitterError::Parsing("CAR roots is empty".to_owned()));
        }
        if header.version != 1 {
            return Err(CarSplitterError::InvalidFile(
                "CAR version 1 is supported only".to_string(),
            ));
        }
        Ok(header)
    }

    pub fn encode(&self) -> Result<Vec<u8>, CarSplitterError> {
        let data = DagCborCodec
            .encode(self)
            .map_err(|e| CarSplitterError::Parsing(e.to_string()))?;
        Ok(data)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        todo!("test");
    }
}
