mod tts;

use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fs,
    io::{stdin, BufRead},
    path::PathBuf,
    process::ExitCode,
    sync::Arc,
};

use fundsp::{
    hacker::{constant, resample, wave64},
    prelude::An,
    sequencer::Sequencer64,
    wave::{Wave64, Wave64Player},
};
use tts::*;

/// Holds configuration values read from config.toml.
#[derive(Deserialize)]
struct Config {
    delay: f64,
    tts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            delay: 10.0,
            tts: true,
        }
    }
}

/// Get the path of the configuration file path.
/// [this executable's directory]/config.toml
fn get_default_config_file_path() -> Option<std::path::PathBuf> {
    match std::env::current_exe() {
        Err(exe_path_error) => {
            println!(
                "[Configuration Loader] Unable to obtain executable directory: {exe_path_error}."
            );
            None
        }
        Ok(exe_path) => match exe_path.parent() {
            None => {
                println!("[Configuration Loader] Unable to obtain executable directory.");
                None
            }
            Some(parent_dir) => Some(parent_dir.join("config.toml")),
        },
    }
}

/// Load the toml configuration from [`path`].
fn load_config_file(path: std::path::PathBuf) -> Option<Config> {
    println!(
        "[Configuration Loader] Loading configuration file \"{}\".",
        path.display()
    );

    match fs::read_to_string(&path) {
        Ok(config_data) => match toml::from_str(&config_data) {
            Err(error) => {
                println!(
                    "[Configuration Loader] Unable to deserialize configuration file: {error}."
                );
                None
            }
            Ok(config) => Some(config),
        },
        Err(read_error) => {
            println!("[Configuration Loader] Unable to open configuration file: {read_error}. Use --ignore-config if this is intentional.");
            None
        }
    }
}

/// Load the toml configuration from [`stdin`].
fn load_config_stdin() -> Option<Config> {
    println!("[Configuration Loader] Loading configuration file from stdin.");
    let mut buffer: Vec<u8> = Vec::new();
    if let Err(read_error) = stdin().lock().read_until(0, &mut buffer) {
        println!("[Configuration Loader] Unable to read from stdin: {read_error}.");
        return None;
    }
    match String::from_utf8(buffer) {
        Ok(data) => match toml::from_str::<Config>(&data) {
            Err(deserialize_error) => {
                println!("[Configuration Loader] Unable to deserialize configuration: {deserialize_error}.");
                None
            }
            Ok(config) => Some(config),
        },
        Err(decode_error) => {
            println!("[Configuration Loader] Unable to decode configuration: {decode_error}.");
            None
        }
    }
}

/// Apply command line configuration options.
fn apply_options(options: &HashMap<String, String>, config: &mut Config) -> bool {
    if let Some(delay) = options.get("delay") {
        if let Ok(delay) = delay.parse() {
            config.delay = delay
        } else {
            println!("[Argument Parser] Invalid f64 `{delay}` for --delay.");
            return false;
        }
    }

    // old --tts --no-tts | new
    // 0   0     0        | 0
    // X   0     1        | 0
    // X   1     X        | 1
    // 1   0     0        | 1
    config.tts = options.contains_key("tts") || (config.tts && !options.contains_key("no-tts"));

    true
}

/// Parse command line arguments.
/// Returns (success, options, arguments)
fn parse_arguments() -> (bool, HashMap<String, String>, Vec<String>) {
    let options_with_arguments = vec!["config", "delay"];
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options: HashMap<String, String> = HashMap::new();
    let mut arguments: Vec<String> = Vec::new();
    let mut option = None;
    for mut token in args {
        if token.starts_with("--") {
            // Remove "--" from the beginning of the option.
            token.remove(0);
            token.remove(0);

            if let Some(name) = option {
                // The previous token was an option with an argument but another argument was specified afterward.
                println!("[Argument Parser] Missing argument for option `{name}`.");
                return (false, options, arguments);
            } else if options_with_arguments.contains(&token.as_str()) {
                // This token is an option with an argument.
                option = Some(token);
            } else {
                // This token is an option without an argument.
                options.insert(token, String::new());
            }
        } else {
            if let Some(name) = option.take() {
                // This token is the argument for an option.
                options.insert(name, token);
            } else {
                // This token is a positional argument.
                arguments.push(token);
            }
        }
    }
    if let Some(name) = option {
        // The last token was an option with an argument but none were given.
        println!("[Argument Parser] Missing argument for option `{name}`.");
        return (false, options, arguments);
    }
    return (true, options, arguments);
}

/// Simpler push interface for [`fundsp::sequencer::Sequencer64`].
fn sequence(
    sequencer: &mut fundsp::sequencer::Sequencer64,
    audio: &Arc<Wave64>,
    in_first_sample: usize,
    in_last_sample: usize,
    out_sec: &mut f64,
) {
    let out_last_sec = *out_sec + (in_last_sample - in_first_sample) as f64 / SAMPLE_RATE;
    sequencer.push(
        *out_sec,
        out_last_sec,
        fundsp::sequencer::Fade::Power,
        0.0,
        0.0,
        Box::new(An(Wave64Player::new(
            &audio,
            0,
            in_first_sample,
            in_last_sample,
            None,
        ))),
    );
    *out_sec = out_last_sec;
}

fn main() -> ExitCode {
    // Get command line arguments and exit 2 on failure.
    let (success, options, arguments) = parse_arguments();
    if !success {
        return 2.into();
    }
    drop(success);

    // Print help.
    if options.contains_key("help") || options.contains_key("h") {
        println!(include_str!("../assets/help.txt"));
        return 0.into();
    }

    // Load config file and exit 1 on failure.
    let Some(mut config) = (if let Some(path) = options.get("config") {
        if path == "-" {
            // --config - => load config from stdin
            load_config_stdin()
        } else {
            // --config FILE => load config from FILE
            load_config_file(path.into())
        }
    } else {
        if options.contains_key("ignore-config") {
            // --ignore-config => don't read the default configuration file config.toml
            Some(Config::default())
        } else {
            // Neither --config nor --ignore-config were specified => load default configuration file config.toml
            get_default_config_file_path().and_then(load_config_file)
        }
    }) else {return 1.into()};

    // Apply command line arguments and exit 2 on failure.
    if !apply_options(&options, &mut config) {
        return 2.into();
    };

    // Obtain input and output file paths and exit 2 on failure.
    let Some(input_path_string) = arguments.get(0) else {
        println!("[Argument Parser] Missing required argument INPUT.");
        return 2.into();
    };
    let Ok(input_path) = PathBuf::from(input_path_string).canonicalize() else {
        println!("[Argument Parser] Invalid input file path {input_path_string}.");
        return 2.into();
    };
    drop(input_path_string);
    let output_path = arguments
        .get(1)
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| input_path.clone());

    println!(
        "[Main] Processing {} into {}.",
        input_path.to_string_lossy(),
        output_path.to_string_lossy()
    );

    // Load input audio and exit 2 on failure.
    println!("[Main] Loading input file.");
    let input_wave = match Wave64::load(&input_path) {
        Ok(wave) => wave,
        Err(load_error) => {
            println!(
                "[Main] Unable to load {}: {load_error}.",
                input_path.to_string_lossy()
            );
            return 2.into();
        }
    };

    // Resample input audio.
    println!(
        "[Main] Resampling from {}Hz to {}Hz.",
        input_wave.sample_rate(),
        SAMPLE_RATE
    );
    let mut input_wave = Wave64::render(
        SAMPLE_RATE,
        input_wave.duration(),
        &mut (constant(input_wave.sample_rate() / SAMPLE_RATE)
            >> resample(wave64(&Arc::new(input_wave), 0, None))),
    );

    println!("[Main] Normalizing audio.");
    input_wave.normalize();

    // Analyze input audio.

    // Recursive RMS calculation. When the RMS value drops below
    // `SILENCE_THRESHOLD` for longer than `config.delay`, add the period to
    // the `silences` vector.
    println!("[Main] Searching for silences.");
    let averaging_coefficient = (-1.0 / (SAMPLE_RATE * 0.5)).exp(); // Half second RMS averaging time. a = exp(-1 / (f * t))
    let silence_threshold_db = -50.0; // RMS values below this threshold are considered silent

    // If the audio is empty, exit 1.
    if input_wave.len() < 1 {
        println!("[Main] Audio is empty.");
        return 1.into();
    }

    let mut mean_squared = input_wave.channel(0)[0] * input_wave.channel(0)[0]; // Initialize the MS value
    let mut silences = Vec::new(); // (start, end) silences

    let mut silence_start = None;
    for (index, sample) in input_wave.channel(0).iter().enumerate() {
        // ms(n) = (1 - a) * x(n) + a * m(n - 1) = x(n) + a * (m(n - 1) - x(n))
        mean_squared = sample * sample + averaging_coefficient * (mean_squared - sample * sample);
        // 20 * log10(sqrt(mean(x ^ 2))) = 10 * log10(mean(x ^ 2))
        let rms_db = 10.0 * mean_squared.log10();
        if rms_db >= silence_threshold_db {
            if let Some(silence_start) = silence_start.take() {
                if index - silence_start > (config.delay * SAMPLE_RATE) as usize {
                    silences.push((silence_start, index))
                }
            }
        } else {
            if silence_start.is_none() {
                silence_start = Some(index);
            }
        }
    }
    if let Some(silence_start) = silence_start.take() {
        silences.push((silence_start, input_wave.len() - 1));
    }

    // Sequence output audio.
    println!("[Main] Sequencing output audio.");
    let audio_arc = Arc::new(input_wave);
    let mut sequencer = Sequencer64::new(true, 1);
    let mut t_out_sec = 0.0;
    let mut t_in_sample = 0;
    for (start_time, end_time) in silences {
        // Add sound before the silence.
        sequence(
            &mut sequencer,
            &audio_arc,
            t_in_sample,
            start_time,
            &mut t_out_sec,
        );

        // Add silence length announcement.
        if config.tts {
            let duration = ((end_time - start_time) as f64 / SAMPLE_RATE).round() as i64;
            let mut tokens = Vec::new();
            if duration >= 60 {
                tokens.extend(tokenize_int(duration / 60));
                tokens.push(TOKEN::MINUTES);
            }
            tokens.extend(tokenize_int(duration % 60));
            tokens.push(TOKEN::SECONDS);
            synthesize(&mut sequencer, &mut t_out_sec, tokens);
        }

        t_in_sample = end_time;
    }
    // Add the last sound if it exists.
    if t_in_sample < audio_arc.len() - 1 {
        sequence(
            &mut sequencer,
            &audio_arc,
            t_in_sample,
            audio_arc.len() - 1,
            &mut t_out_sec,
        );
    }

    // Output audio.
    println!("[Main] Rendering audio sequence.");
    let output_wave = Wave64::render(SAMPLE_RATE, t_out_sec, &mut sequencer);
    println!("[Main] Saving audio to {}.", output_path.to_string_lossy());
    match output_wave.save_wav16(&output_path) {
        Ok(()) => {
            println!("[Main] Done.");
            0.into()
        }
        Err(save_error) => {
            println!("Save error: {save_error}");
            1.into()
        }
    }
}
