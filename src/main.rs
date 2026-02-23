use std::{fs::File, io::BufReader};
use rodio::{Decoder, OutputStream, Sink, Source};

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = File::open("music/Fantasy.mp3").unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let duration = source.total_duration();
    println!("Duration: {:?}", duration);

    sink.append(source);
    sink.sleep_until_end();
}
