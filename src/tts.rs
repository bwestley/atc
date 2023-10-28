use fundsp::wave::Wave64;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

/// Each token represents a soundbite.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum TOKEN {
    SPACE,
    ZERO,
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    ZULU,
    MINUTES,
    SECONDS,
}

/// Sample rate of the embedded tts.wav file.
pub const SAMPLE_RATE: f64 = 44100.0;

/// A mapping of tokens to sample ranges in the embedded tts.wav file.
static TABLE: Lazy<HashMap<TOKEN, (usize, usize)>> = Lazy::new(|| {
    HashMap::from([
        (TOKEN::SPACE, (0, 26800)),
        (TOKEN::ZERO, (36352, 59712)),
        (TOKEN::ONE, (66624, 80832)),
        (TOKEN::TWO, (90816, 107648)),
        (TOKEN::THREE, (115456, 134592)),
        (TOKEN::FOUR, (142656, 162592)),
        (TOKEN::FIVE, (167168, 187840)),
        (TOKEN::SIX, (192544, 218112)),
        (TOKEN::SEVEN, (218656, 240960)),
        (TOKEN::EIGHT, (248288, 268320)),
        (TOKEN::NINE, (274208, 291968)),
        (TOKEN::ZULU, (297088, 329856)),
        (TOKEN::MINUTES, (333824, 372096)),
        (TOKEN::SECONDS, (372096, 410880)),
    ])
});

/// Convert a string to a vector of tokens, skipping any undefined characters.
pub fn tokenize(string: &str) -> Vec<TOKEN> {
    string
        .chars()
        .filter_map(|c| match c {
            ' ' => Some(TOKEN::SPACE),
            '0' => Some(TOKEN::ZERO),
            '1' => Some(TOKEN::ONE),
            '2' => Some(TOKEN::TWO),
            '3' => Some(TOKEN::THREE),
            '4' => Some(TOKEN::FOUR),
            '5' => Some(TOKEN::FIVE),
            '6' => Some(TOKEN::SIX),
            '7' => Some(TOKEN::SEVEN),
            '8' => Some(TOKEN::EIGHT),
            '9' => Some(TOKEN::NINE),
            'Z' => Some(TOKEN::ZULU),
            'm' => Some(TOKEN::MINUTES),
            's' => Some(TOKEN::SECONDS),
            _ => None,
        })
        .collect()
}

/// Convert a positive integer to a vector of tokens.
pub fn tokenize_int(mut number: i64) -> Vec<TOKEN> {
    if number < 0 {
        Vec::new()
    } else if number == 0 {
        vec![TOKEN::ZERO]
    } else {
        let mut tokens = vec![];
        for _ in 0..(number.ilog10() + 1) {
            tokens.push(match number % 10 {
                0 => TOKEN::ZERO,
                1 => TOKEN::ONE,
                2 => TOKEN::TWO,
                3 => TOKEN::THREE,
                4 => TOKEN::FOUR,
                5 => TOKEN::FIVE,
                6 => TOKEN::SIX,
                7 => TOKEN::SEVEN,
                8 => TOKEN::EIGHT,
                9 => TOKEN::NINE,
                _ => unreachable!(),
            });
            number /= 10;
        }
        tokens.reverse();
        tokens
    }
}

/// Add soundbites specified by `tokens` to `sequencer` starting at time `t`.
/// Mutate `t` to the end of the last soundbite.
pub fn synthesize(sequencer: &mut fundsp::sequencer::Sequencer64, t: &mut f64, tokens: Vec<TOKEN>) {
    let mut audio = Wave64::load_slice(include_bytes!("../assets/tts.wav"))
        .expect("Unable to load embedded tts.wav");
    audio.normalize();

    let audio_arc = Arc::new(audio);
    for token in tokens {
        if let Some((start_time, end_time)) = TABLE.get(&token) {
            crate::sequence(sequencer, &audio_arc, *start_time, *end_time, t);
        }
    }
}
