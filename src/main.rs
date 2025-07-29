use crossterm::{
    ExecutableCommand, cursor,
    style::{PrintStyledContent, Stylize},
};
use rand::Rng;
use std::{
    io::{Cursor, Write, stdout},
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink, Source};

const WIDTH: usize = 80;
const HEIGHT: usize = 24;
const PARTICLE_COUNT: usize = 100;

///has to be sorted far away/slow -> close up/fast
const RAIN_PART: [char; 3] = ['\'', '!', '|'];

struct RainParticle {
    /// x and y pos in grid
    position: (f32, f32),
    /// index of particle of RAIN_PART
    character_idx: usize,
    /// determines if redered or not
    alive: bool,
}

fn play_looping_sound(sound_data: &'static [u8]) {
    thread::spawn(move || {
        // Create an audio output stream
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get audio output stream");

        // Decode the MP3 from memory using Cursor
        let cursor = Cursor::new(sound_data.as_ref());
        let source = Decoder::new(cursor).expect("Failed to decode embedded MP3");

        // Create a sink and loop the audio
        let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
        sink.append(source.repeat_infinite());
        loop {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

// \x1B[3J → clears scrollback buffer.
// \x1B[2J → clears the visible screen.

fn clear_terminal() {
    // ANSI escape to clear screen + scrollback
    print!("\x1B[3J\x1B[2J");
    stdout().flush().unwrap();
}

fn draw_rain() {
    let mut stdout = stdout();
    let mut rng = rand::thread_rng();
    //let mut drops = vec![0; WIDTH];
    let mut drops: Vec<RainParticle> = Vec::new();

    //init the drops vector
    for _n in 0..PARTICLE_COUNT {
        drops.push(RainParticle {
            position: (
                rng.gen_range(0.0..(WIDTH as f32)),
                rng.gen_range(0.0..(HEIGHT as f32)),
            ),
            character_idx: rng.gen_range(0..RAIN_PART.len()),
            alive: rng.gen_bool(0.05),
        });
    }

    stdout.execute(cursor::Hide).unwrap();

    loop {
        clear_terminal();

        for drop in &mut drops {
            let past_state: bool = drop.alive;
            drop.alive = rng.gen_bool(0.95);
            //if we have to spawn it anew
            if !past_state && drop.alive {
                drop.position = (
                    rng.gen_range(0.0..(WIDTH as f32)),
                    rng.gen_range(0.0..(HEIGHT as f32)),
                );
            } else {
                drop.position.1 = (drop.position.1
                    + ((drop.character_idx + 1) as f32 / RAIN_PART.len() as f32))
                    % HEIGHT as f32;
            }
            if drop.alive {
                stdout
                    .execute(cursor::MoveTo(
                        drop.position.0 as u16,
                        drop.position.1 as u16,
                    ))
                    .unwrap();
                stdout
                    .execute(PrintStyledContent(RAIN_PART[drop.character_idx].white()))
                    .unwrap();
            }
        }
        stdout
            .execute(cursor::MoveTo(WIDTH as u16, HEIGHT as u16))
            .unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}
fn main() {
    let sound_data = include_bytes!("sounds/light-rain.mp3");
    play_looping_sound(sound_data);
    draw_rain();
}
