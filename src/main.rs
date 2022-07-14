use clap::{ArgGroup, Parser};
use chrono;
use rand::Rng;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

fn iterate_rule(rule: u8, state: String, width: usize, wrapping: bool) -> String {
    let mut next_state = String::new();
    let mut rule_chars: Vec<char> = format!("{:08b}", rule).chars().collect();
    rule_chars.reverse();

    // handle first
    let pattern = if wrapping {
        (&state[(width - 1)..=(width - 1)]).to_string() + &state[..2]
    } else {
       state[..2].to_string()
    };
    let pattern = usize::from_str_radix(&pattern, 2).unwrap();
    next_state.push(rule_chars[pattern]);

    // handle in between
    for i in 1..(width - 1) {
        let pattern = &state[(i - 1)..=(i + 1)];
        let pattern = usize::from_str_radix(&pattern, 2).unwrap();
        next_state.push(rule_chars[pattern]);
    }

    // handle last
    let pattern = if wrapping {
        (&state[(width - 1)..]).to_string() + &state[0..=0]
    } else {
        (&state[(width - 1)..]).to_string() + "0"
    };
    let pattern = usize::from_str_radix(&pattern, 2).unwrap();
    next_state.push(rule_chars[pattern]);

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
// TODO: check if .ppm is already in filepath

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
    #[clap(short, long, value_parser, default_value_t = 30)]
    rule: u8,

    /// Width of simulation [min. 3px], overridden by length of --input <STATE>
    #[clap(short, long, value_parser = parse_size, default_value_t = 256)]
    width: usize,

    /// Height of simulation [min. 3px]
    #[clap(short, long, value_parser = parse_size, default_value_t = 256)]
    height: usize,

    /// Preset initial state [0-3]: {0: Random, 1: Centre, 2: Corners, 3: Alternate}
    #[clap(short, long, value_parser = parse_preset)]
    preset: Option<PresetOption>,

    /// Manual initial state [min. 3 bits], ASCII is assumed if not prefixed with 0b or 0x
    #[clap(short, long, value_name = "STATE", value_parser = parse_input)]
    input: Option<String>,

    /// Output filepath
    #[clap(short, long, value_name = "FILENAME", value_parser)]
    output: Option<PathBuf>,

    /// Disable wrapping during iteration, will assume outside neighbour is 0
    #[clap(long="no-wrapping", action)]
    disable_wrapping: bool
}

#[derive(Debug, Clone)]
enum PresetOption {
    Random = 0,
    Centre = 1,
    Corners = 2,
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
        option if option == PresetOption::Corners as u8 => Ok(PresetOption::Corners),
        option if option == PresetOption::Alternate as u8 => Ok(PresetOption::Alternate),
        _ => Err(format!("option does not exist")),
    }
}

/// Parse command line input for setting initial state. Handles 0b binary, 0x hex, and ASCII strings
fn parse_input(s: &str) -> Result<String, String> {
    let len = s.len();
    // mutable state for each block to fill out, returned at the end.
    // errors along the way will be immediately returned
    let mut state = String::new();
    if (1..=usize::MAX).contains(&len) {
        if len > 2 {
            match &s[..2] {
                "0b" => {
                    for c in s[2..].chars() {
                        if ['0', '1'].contains(&c) {
                            state.push(c);
                        } else {
                            return Err(format!("invalid binary string"))
                        }
                    }
                },
                "0x" => {
                    for c in s[2..].chars() {
                        let hex = u8::from_str_radix(&c.to_string(), 16); // gross
                        match hex {
                            Ok(n) => state.push_str(&format!("{:04b}", n)),
                            Err(_) => return Err(format!("invalid hexadecimal string")),
                        }
                    }
                },
                _ => match ascii_to_binary(s) {
                    Ok(binary_state) => state = binary_state,
                    Err(error) => return Err(error)
                }
            }
        // treat as ascii, note that simply "0b" and "0x" without values will also be ascii
        } else {
            match ascii_to_binary(s) {
                Ok(binary_state) => state = binary_state,
                Err(error) => return Err(error)
            }
        }
    // one character inputs are treated as ascii
    } else {
        return Err(format!("state length is out of range"))
    }
    // 3 is minimum width, as documented. Should really be a const but this is the only occurance
    if state.len() >= 3 {
        Ok(state)
    } else {
        Err(format!("state is too short"))
    }
}

fn ascii_to_binary(s: &str) -> Result<String, String> {
    let mut state = String::new();
    for c in s.as_bytes() {
        match c.is_ascii() {
            true => state.push_str(&format!("{:08b}", c)),
            false => return Err(format!("invalid ascii string")),
        }
    }
    Ok(state)
}

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
            let other_half = if *width % 2 == 0 {half - 1} else {half};
            state = format!("{}1{}", "0".repeat(half), "0".repeat(other_half));
        },
        PresetOption::Corners => {
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
// TODO: Toggable wrapping

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
    let wrapping = if args.disable_wrapping {false} else {true};

    // Simulating
    let mut output: Vec<String> = Vec::new();

    let mut row = initial_state;
    for _ in 0..height {
        output.push(row.clone());
        row = iterate_rule(rule, row, width, wrapping);
    }

    // Outputting
    let _result = match output_ppm(&filepath, width, height, output) {
        Ok(_) => println!("Success! Wrote to {}", filepath.display()),
        Err(error) => panic!("Problem writing file: {:?}", error),
    };
}
