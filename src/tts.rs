use fundsp::{
    prelude::An,
    wave::{Wave64, Wave64Player},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

#[derive(PartialEq, Eq, Hash)]
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

pub static SAMPLE_RATE: f64 = 44100.0;

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

pub fn synthesize(sequencer: &mut fundsp::sequencer::Sequencer64, t: &mut f64, tokens: Vec<TOKEN>) {
    let audio = Arc::new(
        Wave64::load_slice(include_bytes!("../assets/tts.wav"))
            .expect("Unable to load embedded tts.wav"),
    );
    for token in tokens {
        if let Some((start_time, end_time)) = TABLE.get(&token) {
            let t1 = *t + (*end_time as f64 / SAMPLE_RATE) - (*start_time as f64 / SAMPLE_RATE);
            sequencer.push(
                *t,
                t1,
                fundsp::sequencer::Fade::Power,
                0.1,
                0.1,
                Box::new(An(Wave64Player::new(
                    &audio,
                    0,
                    *start_time,
                    *end_time,
                    None,
                ))),
            );
            *t = t1;
        }
    }
}
