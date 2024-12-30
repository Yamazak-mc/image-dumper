use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
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

    let mut size_before = 0;

    for entry in std::fs::read_dir(dir)? {
        let Ok(entry) = entry else {
            continue;
        };

        let filename = entry.file_name().to_string_lossy().to_string();

        let rdr = image::ImageReader::open(entry.path())?;
        if let Ok(img) = rdr.decode() {
            let img_as_bytes = img.to_rgba8().into_raw();

            size_before += {
                let mut buf = Vec::new();
                let mut reader = BufReader::new(std::fs::File::open(entry.path())?);
                reader.read_to_end(&mut buf)?;
                buf.len()
            };

            let mut buf = Vec::new();
            let mut encoder = ZlibEncoder::new(&mut buf, Compression::best());
            encoder.write_all(&img_as_bytes[..])?;
            encoder.finish()?;

            map.insert(filename, BASE64_STANDARD.encode(buf));
        }
    }

    let json = serde_json::to_string(&map)?;
    let size_after = json.as_bytes().len();
    compare_size(size_before, size_after);

    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs();
    let dirname = dir
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let out_file_path = format!("{dirname}-{timestamp:?}.json");

    let mut writer = BufWriter::new(std::fs::File::create(out_file_path)?);
    writer.write_all(&json.as_bytes()[..])?;

    Ok(())
}

fn compare_size(before: usize, after: usize) {
    let p = after as f32 / before as f32 * 100.0;
    println!("\nsize: {before} -> {after} ( {p:.2}% )\n");
}
