use std::io::{self, BufReader, Write};
use std::fs::{self, File, DirEntry};
use std::path::Path;
use hound::{WavWriter, WavSpec};
use rodio::{Decoder, OutputStream, Sink, Source};

fn main() {
    // convert txt to wav lmao
    file_to_wav("target/debug/CliMusic", "music/cli.wav", 1).unwrap();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    read_dirs(Path::new("music"), &|entry| {
        println!("{}", entry.path().display());
    }).unwrap();

    let file = File::open("music/cli.wav").unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let duration = source.total_duration();
    println!("Duration: {:?}", duration);

    sink.append(source);
    sink.sleep_until_end();
}
// lmao
fn file_to_wav(file_path: &str, wav_path: &str, loops: usize) -> std::io::Result<()> {
    let bytes = fs::read(file_path)?;

    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(wav_path, spec).unwrap();
    
    for _ in 0..loops {
        for &b in &bytes {
            let sample = (b as i16 - 128) * 256; // map 0–255 to -32768..32767
            writer.write_sample(sample).unwrap();
        }
    }

    writer.finalize().unwrap();
    Ok(())
}

fn read_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                read_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
