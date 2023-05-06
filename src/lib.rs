use std::io::{Read, Seek};

pub mod error;
pub mod header;
pub mod reader;
pub mod section;
pub mod treewalk;
pub mod writer;

pub enum Strategy {
    Simple,
    Treewalk,
}

pub fn new_splitter<R>(strategy: Strategy, r: R, target_size: usize) -> Box<impl CarSplitter>
where
    R: Read + Seek + 'static,
{
    let splitter = match strategy {
        Strategy::Simple => todo!("Simple splitter not implemented"),
        Strategy::Treewalk => treewalk::TreewalkSplitter::new(r, target_size),
    }
    .unwrap();

    Box::new(splitter)
}

pub trait CarSplitter {
    fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, error::CarSplitterError>;
}
