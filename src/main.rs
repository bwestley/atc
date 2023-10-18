mod tts;

use std::io::{stdin, stdout, BufRead, Write};

use fundsp::{sequencer::Sequencer64, wave::Wave64};
use tts::*;

fn main() {
    let mut sequencer = Sequencer64::new(true, 1);
    let mut duration = 0.0;
    let mut text = String::new();
    print!("Synthesize: ");
    let _ = stdout().flush();
    let _ = stdin().lock().read_line(&mut text);
    synthesize(&mut sequencer, &mut duration, tokenize(&text));
    let wave = Wave64::render(SAMPLE_RATE, duration, &mut sequencer);
    match wave.save_wav16("output.wav") {
        Ok(()) => {}
        Err(save_error) => println!("Save error: {save_error}"),
    };
}
