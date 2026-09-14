use anyhow::*;
use rayon::prelude::*;

type Word = [u8; 5];

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() -> Result<()> {
    let mut guesses: [(Word, f32); GUESS_COUNT] = GUESSES;
    let answers: [Word; ANSWER_COUNT] = ANSWERS;
    
    guesses.par_iter_mut().for_each(|(guess, avg_surprise)| {
        let mut cache: [f32; 243] = [0.0; 243];
        
        for answer in &answers { cache[evaluate(guess, answer) as usize] += 1.0; }
        
        let entropy: f32 = cache
            .iter()
            .filter(|&&count| count > 0.0)          // skip patterns that never occurred
            .map(|&count| {
                let p = count / ANSWER_COUNT as f32;      // probability of this pattern
                -p * p.log2()                      // weighted contribution
            })
            .sum();
        
        *avg_surprise = entropy;
    });

    
    println!("Calculated surprise for all guesses");
    guesses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for i in 0..10 { println!("#{} {} with {} bits of data", i + 1, parse_code(&guesses[i].0), guesses[i].1); }
    
    Ok(())
}

#[inline(always)]
fn evaluate(guess: &Word, answer: &Word) -> u8 {
    let mut counts = [0u8; 26];
    let mut digits = [0u8; 5]; // 0 = Absent, 1 = Present, 2 = Correct

    // Pass 1: mark greens, tally leftover letters in `answer`
    for i in 0..5 {
        // SAFETY: i is always 0..5, well within bounds of both arrays.
        let g = unsafe { *guess.get_unchecked(i) };
        let a = unsafe { *answer.get_unchecked(i) };

        if g == a {
            unsafe { *digits.get_unchecked_mut(i) = 2; }
        } else {
            let idx = (a - b'a') as usize;
            unsafe { *counts.get_unchecked_mut(idx) += 1; }
        }
    }

    // Pass 2: mark yellows
    for i in 0..5 {
        if unsafe { *digits.get_unchecked(i) } != 2 {
            let g = unsafe { *guess.get_unchecked(i) };
            let idx = (g - b'a') as usize;
            let c = unsafe { counts.get_unchecked_mut(idx) };
            if *c > 0 {
                unsafe { *digits.get_unchecked_mut(i) = 1; }
                *c -= 1;
            }
        }
    }

    // Encode directly — no enum, no separate `cached()` call
    digits.iter().fold(0u8, |code, &d| code * 3 + d)
}


fn parse_code(code: &Word) -> String {
    let mut word: String = String::new();
    for c in code {
        word.push(*c as char);
    }
    word
}

#[cfg(test)]
mod tests {
    use crate::{Word, evaluate, parse_word};

    #[test]
    fn eval_test() {
        let guess: Word = parse_word("tarse");
        let answer: Word = parse_word("burst");
        let result = evaluate(&guess, &answer);
        let expected = 105u8;
        assert_eq!(result, expected)
    }
}