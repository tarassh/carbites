use ipld::{Cid, Ipld};
use std::collections::VecDeque;
use std::io::{Read, Seek};

use crate::error::CarSplitterError;
use crate::header::CarHeader;
use crate::reader::CarReader;
use crate::writer::CarWriter;
use crate::CarSplitter;

pub(crate) struct TreewalkSplitter<R> {
    root: Cid,
    target_size: usize,
    reader: CarReader<R>,
    writer: CarWriter,
    pending_blocks: VecDeque<PendingBlock>,
}

impl<R> TreewalkSplitter<R>
where
    R: Read + Seek,
{
    pub(crate) fn new(r: R, target_size: usize) -> Result<TreewalkSplitter<R>, CarSplitterError> {
        let mut reader = CarReader::new(r)?;
        let header = reader.header();

        if header.roots.len() != 1 {
            return Err(CarSplitterError::InvalidFile(
                "multiple roots not allowed".to_owned(),
            ));
        }
        let root = header.roots.first().unwrap().clone();
        let parents = vec![Parent::new(reader.read_section_data(&root)?, root)];

        let writer = new_car(root, parents.clone())?;

        let mut pending_blocks: VecDeque<PendingBlock> = VecDeque::new();
        let ipld = reader.ipld(&root)?;
        match ipld {
            Ipld::Map(_) => match ipld.take("Links") {
                Ok(Ipld::List(links)) => {
                    for link in links.iter() {
                        pending_blocks.push_back(PendingBlock {
                            cid: extract_cid(link)?,
                            parents: parents.clone(),
                        });
                    }
                }
                Ok(_) => {
                    return Err(CarSplitterError::Parsing(
                        "root node does not have links".to_owned(),
                    ))
                }
                Err(e) => {
                    return Err(CarSplitterError::Parsing(e.to_string()));
                }
            },
            Ipld::Bytes(_) => {}
            _ => {
                return Err(CarSplitterError::InvalidFile(
                    "root node is not a map".to_owned(),
                ))
            }
        }

        Ok(TreewalkSplitter {
            root,
            target_size,
            writer,
            reader,
            pending_blocks,
        })
    }

    fn next(&mut self) -> Result<Option<Vec<u8>>, CarSplitterError> {
        loop {
            let pending_block = match self.pending_blocks.pop_front() {
                Some(pending_block) => pending_block,
                None => {
                    if !self.writer.is_empty() {
                        let car = self.writer.flush();
                        return Ok(Some(car));
                    }
                    break;
                }
            };

            let data = self.reader.read_section_data(&pending_block.cid)?;
            let (ready_car, links) = self.add_block(&data, pending_block.cid)?;

            let mut parents = pending_block.parents;

            if !links.is_empty() {
                parents.push(Parent::new(data, pending_block.cid));

                let mut pb = VecDeque::<PendingBlock>::new();
                for link in links.iter() {
                    pb.push_back(PendingBlock {
                        cid: extract_cid(link)?,
                        parents: parents.clone(),
                    });
                }

                pb.append(&mut self.pending_blocks);
                self.pending_blocks = pb;
            }

            if let Some(ready_car) = ready_car {
                self.writer = new_car(self.root, parents)?;
                return Ok(Some(ready_car));
            }
        }

        Ok(None)
    }

    fn add_block(
        &mut self,
        block: &Vec<u8>,
        cid: Cid,
    ) -> Result<(Option<Vec<u8>>, Vec<Ipld>), CarSplitterError> {
        let mut car_ready = false;
        if block.len() + self.writer.len() > self.target_size {
            car_ready = true;
        }

        self.writer.write(cid, &block)?;

        let ipld = self.reader.ipld(&cid)?;
        let links = ipld.take("Links").unwrap_or(Ipld::List(Vec::new()));
        match (car_ready, links) {
            (true, Ipld::List(links)) => Ok((Some(self.writer.flush()), links)),
            (false, Ipld::List(links)) => Ok((None, links)),
            _ => Err(CarSplitterError::Parsing(
                "root node does not have links".to_owned(),
            )),
        }
    }
}

impl<R> CarSplitter for TreewalkSplitter<R>
where
    R: Read + Seek,
{
    fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, CarSplitterError> {
        TreewalkSplitter::next(self)
    }
}

fn extract_cid(ipld: &Ipld) -> Result<Cid, CarSplitterError> {
    let hash = ipld.clone().take("Hash");
    match hash {
        Ok(Ipld::Link(cid)) => Ok(cid),
        Ok(_) => Err(CarSplitterError::InvalidFile(
            "Root node does not have links".to_owned(),
        )),
        Err(e) => Err(CarSplitterError::Parsing(e.to_string())),
    }
}

fn new_car(root: Cid, parents: Vec<Parent>) -> Result<CarWriter, CarSplitterError> {
    let header = CarHeader::new(vec![root]);

    let mut writer = CarWriter::new(header);
    writer.write_header()?;

    for parent in parents {
        writer.write(parent.cid, &parent.data)?;
    }

    Ok(writer)
}

#[derive(Debug, Clone)]
struct Parent {
    pub data: Vec<u8>,
    pub cid: Cid,
}

impl Parent {
    pub fn new(data: Vec<u8>, cid: Cid) -> Parent {
        Parent { data, cid }
    }
}

struct PendingBlock {
    pub parents: Vec<Parent>,
    pub cid: Cid,
}
