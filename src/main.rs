
use std::fs::File;
use std::io::prelude::*;
use min_max::*;

// fn iterate_rule(rule: &u8, state: u128) -> u128 {
//     let mut next_state = 0u128;
//     for i in 1..127 {
//         let pattern = (state >> i) & 0b111;

//     }
//     next_state
// }

fn iterate_rule(rule: &u8, state: String) -> String {
    let mut next_state = String::new();
    let curr_state: Vec<char> = state.chars().collect();
    let mut rule_chars: Vec<char> = format!("{rule:08b}").chars().collect();
    rule_chars.reverse();
    let len = curr_state.len();

    // println!("{:#?}", rule_chars);
    //

    for i in 0..len {
        let mut pattern = 0u8;
        let left = max!(i as i8 - 1, 0) as usize;
        let right = (i + 1) % len;
        // println!("{}, {}, {}", i, curr_state[left], curr_state[right]);
        pattern += (curr_state[left] as u8 - 0x30) * 4;
        pattern += (curr_state[i] as u8 - 0x30) * 2;
        pattern += (curr_state[right] as u8 - 0x30) * 1;

        // println!("{}, {}", rule_chars[pattern as usize], pattern);
        next_state.push(rule_chars[pattern as usize]);
    }

    next_state

}

fn output_ppm(filepath: &str, width: usize, height: usize, data: Vec<String>) -> std::io::Result<()>{
    let mut file = File::create("output.ppm")?;
    file.write_all(b"P1\n")?;
    file.write_all(format!("{} {}\n", width.to_string(), height.to_string()).as_bytes())?;
    for i in 0..data.len() {
        let chars: Vec<char> = data[i].chars().collect();
        let mut line = String::new();
        for j in 0..chars.len() {
            line.push(chars[j]);
            line.push(' ');
        }
        line.push('\n');
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

fn main() {
    println!("Hello, world!");

    let initial = 0b1u128 << 64;
    let initial = format!("{initial:0128b}");
    let mut output: Vec<String> = vec![initial.clone()];

    let mut row = initial;
    for i in 0..256 {
        // println!("{}", iterate_rule(&30, String::from("000010000")));
        row = iterate_rule(&135, row);
        output.push(row.clone());
    }

    let a = output_ppm("output/output.ppm", 128, output.len(), output);
}
