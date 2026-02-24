use std::io::{self, BufReader};
use std::fs::{self, File, DirEntry};
use std::path::{Path, PathBuf};
use std::time::Duration;
use hound::{WavWriter, WavSpec};
use rodio::{Decoder, OutputStream, Sink, Source};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

struct Idk {
    files: Vec<PathBuf>,
    selected: usize,
}

impl Idk {
    fn next(&mut self) {
        if self.selected + 1 < self.files.len() { self.selected += 1; }
    }
    fn previous(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("music")?;
    enable_raw_mode()?;

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let mut current_sink: Option<Sink> = None;

    // scan music directory
    let mut idk = Idk {
        files: fs::read_dir("music")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect(),
        selected: 0,
    };

    loop {
        // clear screen
        print!("\x1B[2J\x1B[1;1H");

        // draw files
        for (i, file) in idk.files.iter().enumerate() {
            let name = file.file_name().unwrap().to_string_lossy();
            if i == idk.selected {
                println!(">>> {}", name);
            } else {
                println!("    {}", name);
            }
        }

        // handle keys
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => idk.previous(),
                    KeyCode::Down => idk.next(),
                    KeyCode::Esc => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                    }
                    KeyCode::Enter => {
                        // stop previous song
                        if let Some(old_sink) = current_sink.take() {
                            old_sink.stop();
                        }

                        let selected_path = &idk.files[idk.selected];
                        let play_path: PathBuf;

                        if is_supported(selected_path) {
                            play_path = selected_path.clone();
                        } else {
                            // convert unsupported → temp.wav
                            let wav_path = Path::new("music/temp.wav");
                            if let Err(e) = file_to_wav(
                                selected_path.to_str().unwrap(),
                                wav_path.to_str().unwrap(),
                                1
                            ) {
                                println!("Failed to convert {}: {:?}", selected_path.display(), e);
                                continue;
                            }
                            play_path = wav_path.to_path_buf();
                        }

                        // play safely
                        match File::open(&play_path)
                            .and_then(|f| Decoder::new(BufReader::new(f))
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)))
                        {
                            Ok(source) => {
                                let sink = Sink::try_new(&stream_handle)?;
                                sink.append(source);
                                current_sink = Some(sink);
                            }
                            Err(e) => println!("Cannot play {}: {:?}", play_path.display(), e),
                        }
                    }
                    KeyCode::Char('q') => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}

fn is_supported(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp3") | Some("wav") | Some("flac") | Some("ogg") => true,
        _ => false,
    }
}
// lmao
fn file_to_wav(input: &str, output: &str, loops: usize) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(input);
    
    if is_supported(path) {
        // real audio, decode normally
        let file = File::open(input)?;
        let decoder = Decoder::new(BufReader::new(file))?;

        let spec = WavSpec {
            channels: decoder.channels() as u16,
            sample_rate: decoder.sample_rate(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = WavWriter::create(output, spec)?;

        for _ in 0..loops {
            let file = File::open(input)?;
            let decoder = Decoder::new(BufReader::new(file))?;
            for sample in decoder.convert_samples::<i16>() {
                writer.write_sample(sample)?;
            }
        }

        writer.finalize()?;
    } else {
        // unsupported → generate valid WAV from raw bytes
        let bytes = std::fs::read(input)?;
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = WavWriter::create(output, spec)?;

        for _ in 0..loops {
            for &b in &bytes {
                let sample = (b as i16 - 128) * 256; // map 0–255 to -32768..32767
                writer.write_sample(sample)?;
            }
        }

        writer.finalize()?;
    }

    Ok(())
}

fn read_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
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
