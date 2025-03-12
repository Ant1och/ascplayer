extern crate ffmpeg_next as ffmpeg;

use A2VConverter::AudioVideoConverter;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, DisableLineWrap};
use crossterm::{cursor, execute};
use ffmpeg::Rational;
use ffmpeg::format::{Pixel, input};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use rodio::OutputStreamHandle;
use rodio::{Decoder, OutputStream, source::Source};
use spin_sleep::{SpinSleeper, SpinStrategy};
use std::env::args;
use std::fs::{self, File};
use std::io::{BufReader, Stdout, stdout};
use std::process::exit;

// pub const CHARS: &str = " .:-=+*r[A#%$M@";
// pub const CHARS: &str = r##" .-:=!*/#%@"##;
// pub const CHARS: &str = r##" .'`^",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"##;
// pub const CHARS: &str = " ````....---''':::___,,,^^^===;;;>>><<++!!rrcc**//zz??ssLLTTvv))J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";
pub const CHARS: &str = r##" ░▒▓█"##;
// pub const CHARS: &str = " .=!*//A#@";
pub const HEIGHT: u32 = 240;
pub const WIDTH: u32 = 320;

fn main() -> Result<(), ffmpeg::Error> {
    let Some(file_path) = args().nth(1) else {
        wrong_arguments_tip();
        exit(0);
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut player = Player::new(file_path, CHARS, stream_handle);

    player.play()?;

    Ok(())
}

pub struct Player {
    file_path: String,
    fps: f64,
    char_set: &'static str,
    out: Stdout,
    sleeper: SpinSleeper,
    stream_handle: OutputStreamHandle,
}

impl Player {
    fn new(file_path: String, char_set: &'static str, stream_handle: OutputStreamHandle) -> Self {
        ffmpeg::init().unwrap();
        Player {
            file_path,
            fps: 0.,
            char_set,
            out: stdout(),
            sleeper: SpinSleeper::new(1).with_spin_strategy(SpinStrategy::YieldThread),
            stream_handle,
        }
    }

    fn play_frames(&mut self, decoder: &mut ffmpeg::decoder::Video) -> Result<(), ffmpeg::Error> {
        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            2 * WIDTH,
            HEIGHT,
            Flags::BILINEAR,
        )?;

        let mut decoded = Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;
            // println!("frame: {:?}", rgb_frame.data(0).len());
            // println!("{}, {}", decoder.width(), decoder.height());
            let frame = self.frame_to_string(&rgb_frame, 2 * WIDTH);
            self.render_frame(frame);
            self.sleeper.sleep_s(self.fps);
            clear_terminal(&mut self.out);
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: String) {
        execute!(self.out, Print(frame),).expect("render video frame");
        // println!("{}", frame);
    }

    fn set_fps(&mut self, fps_ratio: Rational) {
        self.fps = fps_ratio.numerator() as f64 / fps_ratio.denominator() as f64 / 1000. / 3.5;
    }

    fn play_video(&mut self) -> Result<(), ffmpeg::Error> {
        let mut ictx = input(self.file_path.as_str())?;

        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        self.set_fps(decoder.frame_rate().unwrap());

        let mut stdout = stdout();
        execute!(stdout, cursor::Hide, DisableLineWrap, Clear(ClearType::All),)
            .expect("prepare terminal for starting video playback");

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                // println!("{:?}", self.next_frame(&mut decoder)?);
                self.play_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        self.play_frames(&mut decoder)?;

        Ok(())
    }

    fn create_audio_file(&self) {
        let _ = fs::remove_file("audio.wav");

        AudioVideoConverter::convert_video_to_audio(self.file_path.as_str(), "audio.wav").unwrap();
    }

    fn play_audio(&self) {
        self.create_audio_file();
        let audio_file = File::open("audio.wav").expect("File should exist after conversion");
        let audio_buf_file = BufReader::new(audio_file);
        let source = Decoder::new(audio_buf_file).unwrap();

        self.stream_handle
            .play_raw(source.convert_samples())
            .expect("start audio playback");
    }

    fn play(&mut self) -> Result<(), ffmpeg::Error> {
        self.play_audio();
        self.play_video()?;
        Ok(())
    }

    fn frame_to_string(&self, frame: &Video, width: u32) -> String {
        let mut res: String = "".to_string();
        for (i, lum) in raw_frame_to_lum(frame).iter().enumerate() {
            let lookup_idx = self.char_set.chars().count() * *lum as usize / (u8::MAX as usize + 1);
            let char = self.char_set.chars().collect::<Vec<char>>()[lookup_idx];
            res += char.to_string().as_str();
            if (i + 1) % width as usize == 0 {
                res += "\n";
            }
        }

        res
    }
}

fn wrong_arguments_tip() {
    println!("Usage: ascplayer <path_to_video>");
}

fn raw_frame_to_lum(frame: &Video) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    let data = divide_in_triples_u8(frame.data(0).to_vec());
    for pixel in &data {
        res.push((((*pixel)[0] as u16 + (*pixel)[1] as u16 + (*pixel)[2] as u16) / 3) as u8);
    }
    res
}

fn divide_in_triples_u8(list: Vec<u8>) -> Vec<Vec<u8>> {
    let res: Vec<Vec<u8>> = list
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]].to_vec())
        .collect();
    res
}

fn clear_terminal(out: &mut Stdout) {
    execute!(
        out,
        Clear(ClearType::FromCursorDown),
        cursor::MoveTo(0, 0),
        Clear(ClearType::Purge),
        Clear(ClearType::UntilNewLine),
    )
    .expect("clear terminal");
}
