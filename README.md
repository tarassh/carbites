# carbites

Chunking for [CAR files](https://ipld.io/specs/transport/car/). Split a single CAR into multiple CARs.

Original implementation in [go-carbites](https://github.com/alanshaw/go-carbites).

## Usage

Carbites supports treewalk strategy:

**Treewalk** - walks the DAG to pack sub-graphs into each CAR file that is output. Every CAR file has the _same_ root CID but contains a different portion of the DAG. The DAG is traversed from the root node and each block is decoded and links extracted in order to determine which sub-graph to include in each CAR.

```rust
use carbites::{new_splitter, CarSplitter, Strategy};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("examples/test.car")?;
    let mut splitter = new_splitter(Strategy::Treewalk, file, 1024);

    let mut i = 0;
    while let Some(chunk) = splitter.next_chunk()? {
        let mut file = std::fs::File::create(format!("target/chunk-{}.car", i))?;
        file.write(&chunk)?;
        i += 1;
    }

    Ok(())
}
```

## Todo

- [ ] Add support for simple strategy (see below)

**Simple** - fast but naive, only the first CAR output has a root CID, subsequent CARs have a placeholder "empty" CID. The first CAR output has roots in the header, subsequent CARs have an empty root CID [`bafkqaaa`](https://cid.ipfs.io/#bafkqaaa) as [recommended](https://ipld.io/specs/transport/car/carv1/#number-of-roots).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.