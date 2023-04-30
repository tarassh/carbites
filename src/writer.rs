use crate::{error::CarSplitterError, header::CarHeader};
use integer_encoding::VarIntWriter;
use ipld::Cid;

pub(crate) struct CarWriter {
    inner: Vec<u8>,
    header: CarHeader,
}

impl CarWriter {
    pub fn new(header: CarHeader) -> Self {
        Self {
            inner: Vec::new(),
            header,
        }
    }

    pub fn write_header(&mut self) -> Result<(), CarSplitterError> {
        let head = self.header.encode()?;
        self.inner.write_varint(head.len())?;
        self.inner.extend(&head);
        Ok(())
    }

    pub fn write<T>(&mut self, cid: Cid, data: T) -> Result<(), CarSplitterError>
    where
        T: AsRef<[u8]>,
    {
        let mut cid_buff: Vec<u8> = Vec::new();
        cid.write_bytes(&mut cid_buff)
            .map_err(|e| CarSplitterError::Parsing(e.to_string()))?;
        let data = data.as_ref();
        let sec_len = data.len() + cid_buff.len();
        self.inner.write_varint(sec_len)?;
        self.inner.extend(&cid_buff[..]);
        self.inner.extend(data);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn flush(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.inner)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        todo!("test")
    }
}
