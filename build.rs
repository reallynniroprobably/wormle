use std::env;
use std::fs;
use std::path::Path;
use rayon::prelude::*;

fn main() {
    // Only rerun this script if the word lists actually change.
    println!("cargo:rerun-if-changed=data/guesses.txt");
    println!("cargo:rerun-if-changed=data/answers.txt");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs"); // no leading slash

    let guesses = fs::read_to_string("data/guesses.txt").expect("Failed to read guesses");
    let guesses_result: Vec<[u8; 5]> = guesses.par_lines().map(parse_word).collect();

    let answers = fs::read_to_string("data/answers.txt").expect("Failed to read answers");
    let answers_result: Vec<[u8; 5]> = answers.par_lines().map(parse_word).collect();

    // Both consts built into one string, one write — nothing gets clobbered.
    // Fixed-size arrays (not Vec) so these are genuinely const-compatible.
    let output = format!(
        "pub const GUESSES: [[u8; 5]; {}] = {:?};\n\
         pub const ANSWERS: [[u8; 5]; {}] = {:?};\n\
         pub const ANSWER_COUNT: usize = {};\n\
         pub const GUESS_COUNT: usize = {};\n\
         pub fn parse_word(line: &str) -> [u8; 5] {{\n\
         \tlet mut word = ['0' as u8; 5];\n\
         \tfor (i, c) in line.char_indices() {{\n\
         \t\tword[i] = c as u8;\n\
         \t}}\n\
         \tword\n\
         }}\n",
        guesses_result.len(), guesses_result,
        answers_result.len(), answers_result,
        answers_result.len(),
        guesses_result.len()
    );

    fs::write(&dest_path, output).unwrap();
}

fn parse_word(line: &str) -> [u8; 5] {
    let mut word = ['0' as u8; 5];
    for (i, c) in line.char_indices() {
        word[i] = c as u8;
    }
    word
}