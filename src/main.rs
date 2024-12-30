use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time,
};

use base64::prelude::*;
use clap::Parser;
use flate2::{write::ZlibEncoder, Compression};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommands {
    ReadDir { path: PathBuf },
}

impl Subcommands {
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::ReadDir { path } => read_dir(path),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    cli.subcommands.run()
}

fn read_dir(dir: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let dir = dir.as_ref();

    let mut map = HashMap::new();

    for entry in std::fs::read_dir(dir)? {
        let Ok(entry) = entry else {
            continue;
        };

        let filename = entry.file_name().to_string_lossy().to_string();

        let rdr = image::ImageReader::open(entry.path())?;
        if let Ok(img) = rdr.decode() {
            let img_as_bytes = img.to_rgba8().into_raw();

            let mut buf = Vec::new();
            let mut encoder = ZlibEncoder::new(&mut buf, Compression::best());
            encoder.write_all(&img_as_bytes[..])?;
            encoder.finish()?;

            map.insert(filename, BASE64_STANDARD.encode(buf));
        }
    }

    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs();
    let dirname = dir
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let out_file_path = format!("{dirname}-{timestamp:?}.json");
    let writer = BufWriter::new(std::fs::File::create(out_file_path)?);
    serde_json::to_writer_pretty(writer, &map)?;

    Ok(())
}
