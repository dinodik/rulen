use clap::{ArgGroup, Parser};
use chrono;
use rand::Rng;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

fn iterate_rule(rule: u8, state: String, width: usize) -> String {
    let mut next_state = String::new();
    let curr_state: Vec<char> = state.chars().collect();
    let mut rule_chars: Vec<char> = format!("{:08b}", rule).chars().collect();
    rule_chars.reverse();

    // handle first
    let pattern = curr_state[width - 1].to_string() + &state[..2];
    let pattern = usize::from_str_radix(&pattern, 2).unwrap();
    next_state.push(rule_chars[pattern]);

    // handle rest
    for i in 1..width {
        let mut pattern = String::new();
        for j in 0..3 {
            pattern.push(curr_state[(i + j - 1) % width]);
        }
        let pattern = usize::from_str_radix(&pattern, 2).unwrap();
        next_state.push(rule_chars[pattern]);
    }

    next_state
}

fn output_ppm(filepath: &PathBuf, width: usize, height: usize, data: Vec<String>) -> std::io::Result<()> {
    let mut file = File::create(filepath)?;
    let header = format!("{}\n{} {}\n", "P1", width.to_string(), height.to_string());
    file.write_all(header.as_bytes())?;

    for i in 0..height {
        let mut line = data[i]
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        line.push('\n');
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

/// A program to simulate Wolfram's Rule N cellular automaton and output result into a .ppm file.
#[derive(Parser, Debug)]
#[clap(about)]
#[clap(group(
            ArgGroup::new("setting")
                .required(true)
                .args(&["preset", "input"])
        ))]
struct Args {
    /// Rule [0-255]
    #[clap(short, long, value_parser, default_value_t = 135)]
    rule: u8,

    /// Width of simulation [min. 3]
    #[clap(short, long, value_parser = parse_size, default_value_t = 256)]
    width: usize,

    /// Height of simulation [min. 3]
    #[clap(short, long, value_parser = parse_size, default_value_t = 256)]
    height: usize,

    /// Preset initial state [0-3]: {0: Random, 1: Centre, 2: Edges, 3: Alternate}
    #[clap(short, long, value_parser = parse_preset)]
    preset: Option<PresetOption>,

    /// Manual initial state, ASCII is assumed if not prefixed with 0b or 0x
    #[clap(short, long, value_name = "STATE", value_parser = parse_input)]
    input: Option<String>,

    /// Output filename
    #[clap(short, long, value_name = "FILENAME", value_parser)]
    output: Option<PathBuf>
}

#[derive(Debug, Clone)]
enum PresetOption {
    Random = 0,
    Centre = 1,
    Edges = 2,
    Alternate = 3,
}

fn parse_size(s: &str) -> Result<usize, String> {
    let size: usize = s
        .parse()
        .map_err(|_| format!("`{}` isn't a valid number", s))?;
    if size >= 3 {
        Ok(size)
    } else {
        Err(format!("size is out of range, must be at least 3"))
    }
}

fn parse_preset(s: &str) -> Result<PresetOption, String> {
    let option: u8 = s
        .parse()
        .map_err(|_| format!("`{}` isn't a valid number", s))?;
    match option {
        option if option == PresetOption::Random as u8 => Ok(PresetOption::Random),
        option if option == PresetOption::Centre as u8 => Ok(PresetOption::Centre),
        option if option == PresetOption::Edges as u8 => Ok(PresetOption::Edges),
        option if option == PresetOption::Alternate as u8 => Ok(PresetOption::Alternate),
        _ => Err(format!("option does not exist")),
    }
}

fn parse_input(s: &str) -> Result<String, String> {
    let input = s.to_string();
    let len = input.len();
    if (1..=usize::MAX).contains(&len) {
        // Ok(State {cells: (&input[2..]).to_string(), width: len})
        Ok((&input[2..]).to_string())
    } else {
        return Err(format!("state length is out of range"))
    }
}

// minimums:
// 3 bits   "0b111"
// 1 hex    "0xf"
// 1 ascii  "z"

// #[derive(Debug, Clone)]
// struct State {
//     cells: String,
//     width: usize,
// }

fn generate_preset_state(option: &PresetOption, width: &usize) -> String {
    let mut state = String::new();
    match option {
        PresetOption::Random => {
            let mut rng = rand::thread_rng();
            for _ in 0..*width {
                let p: f32 = rng.gen();
                state.push(if p < 0.5 {'0'} else {'1'});
            }
        },
        PresetOption::Centre => {
            let half = *width / 2;
            state = format!("{}1{}", "0".repeat(half), "0".repeat(half - 1));
        },
        PresetOption::Edges => {
            state = format!("1{}1", "0".repeat(*width - 2))
        },
        PresetOption::Alternate => {
            for i in 0..*width {
                state.push(if i % 2 == 0 {'0'} else {'1'});
            }
        },
    }
    state
}

// TODO: More colours with higher base states? Try with trinary first

fn main() {
    let args = Args::parse();

    // Retrieving arguments
    let rule: u8 = args.rule;
    let height: usize = args.height;
    let initial_state = if let Some(state) = args.input {
        state.to_string()
    } else {
        generate_preset_state(&args.preset.unwrap(), &args.width)
    };
    let width = initial_state.len();
    let filepath = if let Some(file) = args.output {
        file
    } else {
        PathBuf::from(format!("output/output-{}.ppm", chrono::offset::Local::now()))
    };

    let mut output: Vec<String> = Vec::new();

    let mut row = initial_state;
    for _ in 0..height {
        output.push(row.clone());
        row = iterate_rule(rule, row, width);
    }

    let _result = match output_ppm(&filepath, width, height, output) {
        Ok(_) => println!("Success! Wrote to {}", filepath.display()),
        Err(error) => panic!("Problem writing file: {:?}", error),
    };
}
