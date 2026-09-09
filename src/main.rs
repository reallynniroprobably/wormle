use anyhow::*;
use rayon::prelude::*;

type Word = [u8; 5];

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() -> Result<()> {
    let mut guesses: [(Word, f32); GUESS_COUNT] = std::array::from_fn(|i| {
        (GUESSES[i], 0.0_f32)
    });
    let answers: [Word; ANSWER_COUNT] = ANSWERS;
    guesses.par_iter_mut().for_each(|(guess, avg_surprise)| {
        let mut cache: [f32; 243] = [0.0; 243];
        
        for answer in &answers {
            let states = evaluate(guess, answer);
            cache[cached(&states) as usize] += 1.0;
        }
        // println!("Done all answers for guess {:?}", guess);
        
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LetterState {
    Correct,
    Present,
    Absent
}

fn evaluate(guess: &Word, answer: &Word) -> [LetterState; 5] {
    let mut result: [LetterState; 5] = [LetterState::Absent; 5];
    let mut linked: [Option<usize>; 5] = [None; 5];

    // Get correct ones first
    for (i, c) in guess.iter().enumerate() {
        if answer[i] == *c {
            result[i] = LetterState::Correct;
            linked[i] = Some(i);
        }
    }

    for (i, c) in guess.iter().enumerate() {
        if let LetterState::Correct = result[i] { continue; }

        result[i] = LetterState::Absent;
        for (ai, ac) in answer.iter().enumerate() {
            if ac == c && linked[ai] == None {
                result[i] = LetterState::Present;
                linked[ai] = Some(i);
                break;
            }
        }
    }
    
    result
}

fn cached(states: &[LetterState; 5]) -> u8 {
    let mut code = 0u8;
    
    for s in states {
        let digit: u8 = match s {
            LetterState::Absent => 0,
            LetterState::Present => 1,
            LetterState::Correct => 2,
        };
        code = code * 3 + digit;
    }
    code
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
    use crate::{LetterState, Word, evaluate, parse_word};

    #[test]
    fn eval_test() {
        let guess: Word = parse_word("tarse");
        let answer: Word = parse_word("burst");
        let result = evaluate(&guess, &answer);
        let expected = [
            LetterState::Present,
            LetterState::Absent,
            LetterState::Correct,
            LetterState::Correct,
            LetterState::Absent
        ];
        assert_eq!(result, expected)
    }
}