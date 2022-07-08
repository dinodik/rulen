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

fn output_ppm(filepath: &str, width: usize, height: usize, data: Vec<String>) -> std::io::Result<()> {
    let mut file = File::create(filepath)?;
    let header = format!("{}\n{} {}\n", "P1", width.to_string(), height.to_string());
    file.write_all(header.as_bytes())?;

    for i in 0..height {
        let line = data[i]
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn main() {
    // const input: String = ""
    let height: usize = 128;
    let rule: u8 = 30;

    let initial_state = format!("{:0256b}", 1u128 << 127);
    let width: usize = initial_state.len();

    let mut output: Vec<String> = Vec::new();

    let mut row = initial_state;
    for _ in 0..height {
        output.push(row.clone());
        row = iterate_rule(rule, row, width);
    }

    let _result = match output_ppm("output/output.ppm", width, height, output) {
        Ok(_) => println!("Success!"),
        Err(error) => panic!("Problem writing file: {:?}", error),
    };
}
