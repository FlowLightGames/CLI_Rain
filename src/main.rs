use anyhow::Result;
use crossterm::{
    ExecutableCommand, cursor, execute,
    style::{Color, PrintStyledContent, Stylize},
    terminal,
};
use rand::{Rng, rngs::ThreadRng};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    io::{Cursor, Write, stdout},
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink, Source};

/// has to be sorted far away/slow -> close up/fast
const RAIN_PART: [char; 3] = ['\'', '!', '|'];

struct RainParticle {
    /// x and y pos in grid
    position: (f32, f32),
    /// index of particle of RAIN_PART
    character_idx: usize,
    /// determines if redered or not
    alive: bool,
}

struct CursorGuard;

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let _ = execute!(
            stdout(),
            terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
    }
}

fn play_looping_sound(sound_data: &'static [u8], run_flag: Arc<AtomicBool>) {
    thread::spawn(move || {
        // Create an audio output stream
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get audio output stream");

        // Decode the MP3 from memory using Cursor
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor).expect("Failed to decode embedded MP3");

        // Create a sink and loop the audio
        let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
        sink.append(source.repeat_infinite());
        while run_flag.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

// \x1B[3J → clears scrollback buffer.
// \x1B[2J → clears the visible screen.

fn clear_terminal() -> Result<()> {
    // ANSI escape to clear screen + scrollback
    print!("\x1B[3J\x1B[2J");
    stdout().flush()?;
    Ok(())
}

fn create_particles(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<RainParticle> {
    let mut drops: Vec<RainParticle> = Vec::new();
    let particle_count = width as f32 * height as f32 * 0.05;
    for _n in 0..particle_count as usize {
        drops.push(RainParticle {
            position: (
                rng.gen_range(0.0..(width as f32)),
                rng.gen_range(0.0..(height as f32)),
            ),
            character_idx: rng.gen_range(0..RAIN_PART.len()),
            alive: rng.gen_bool(0.05),
        });
    }
    drops
}
fn create_color_map() -> Vec<Color> {
    let mut out: Vec<Color> = Vec::with_capacity(RAIN_PART.len());
    let interval = 1.0 / (RAIN_PART.len() + 1) as f32;
    for n in 0..RAIN_PART.len() {
        let shade = (((n as f32 * interval) + interval).clamp(0.0, 1.0) * 255.0) as u8;
        out.push(Color::Rgb {
            r: shade,
            g: shade,
            b: shade,
        });
    }
    out
}

fn draw_rain(run_flag: &AtomicBool) -> Result<()> {
    let (mut width, mut height) = terminal::size()?;
    let mut stdout = stdout();
    let mut rng = rand::thread_rng();
    let color_map = create_color_map();
    let mut drops: Vec<RainParticle> = create_particles(width as usize, height as usize, &mut rng);

    stdout.execute(cursor::Hide)?;

    while run_flag.load(Ordering::SeqCst) {
        let (new_width, new_height) = terminal::size()?;
        if new_height != height || new_width != width {
            drops = create_particles(width as usize, height as usize, &mut rng);
            width = new_width;
            height = new_height;
        }

        clear_terminal()?;

        for drop in &mut drops {
            let past_state: bool = drop.alive;
            drop.alive = rng.gen_bool(0.95);
            //if we have to spawn it anew
            if !past_state && drop.alive {
                drop.position = (
                    rng.gen_range(0.0..(width as f32)),
                    rng.gen_range(0.0..(height as f32)),
                );
            } else {
                drop.position.1 = (drop.position.1
                    + ((drop.character_idx + 1) as f32 / RAIN_PART.len() as f32))
                    % height as f32;
            }
            if drop.alive {
                stdout.execute(cursor::MoveTo(
                    drop.position.0 as u16,
                    drop.position.1 as u16,
                ))?;
                stdout.execute(PrintStyledContent(
                    RAIN_PART[drop.character_idx].with(color_map[drop.character_idx]),
                ))?;
            }
        }
        stdout.execute(cursor::MoveTo(width, height))?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
fn main() -> Result<()> {
    // Ensure cursor is restored on exit
    let _guard = CursorGuard;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let sound_data = include_bytes!("sounds/light-rain.mp3");
    play_looping_sound(sound_data, running.clone());
    draw_rain(&running)?;

    Ok(())
}
