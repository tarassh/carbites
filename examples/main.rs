use carbites;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("examples/test.car")?;
    let mut splitter = carbites::new_splitter(carbites::Strategy::Treewalk, file, 1024);

    let mut i = 0;
    while let Some(chunk) = splitter.next_chunk()? {
        let mut file = std::fs::File::create(format!("target/chunk-{}.car", i))?;
        file.write(&chunk)?;
        i += 1;
    }

    Ok(())
}
