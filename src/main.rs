use anyhow::*;
use rayon::prelude::*;

type Word = [char; 5];

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() -> Result<()> {
    let mut guesses: Vec<(Word, f32)> = {
        let guesses = GUESSES
            .into_iter()
            .map(|guess| (guess, 0.0))
            .collect();

        guesses
    };
    let answers: Vec<Word> = ANSWERS.to_vec();

    guesses.par_iter_mut().for_each(|(guess, avg_surprise)| {
        let mut cache: [f32; 243] = [0.0; 243];
        
        for answer in &answers {
            let states = evaluate(guess, answer);
            cache[cached(&states)] += 1.0;
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

    for i in 0..10 { println!("#{} {:?} with {} bits of data", i + 1, guesses[i].0, guesses[i].1); }
    
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LetterState {
    Correct(char),
    Present(char, usize),
    Absent(char)
}

fn evaluate(guess: &Word, answer: &Word) -> [LetterState; 5] {
    let mut result: [LetterState; 5] = [LetterState::Absent('0'); 5];
    let mut linked: [Option<usize>; 5] = [None; 5];

    // Get correct ones first
    for (i, c) in guess.iter().enumerate() {
        if answer[i] == *c {
            result[i] = LetterState::Correct(*c);
            linked[i] = Some(i);
        }
    }

    for (i, c) in guess.iter().enumerate() {
        if let LetterState::Correct(_) = result[i] { continue; }

        result[i] = LetterState::Absent(*c);
        for (ai, ac) in answer.iter().enumerate() {
            if ac == c && linked[ai] == None {
                result[i] = LetterState::Present(*c, ai);
                linked[ai] = Some(i);
                break;
            }
        }
    }
    
    result
}

fn cached(states: &[LetterState; 5]) -> usize {
    let mut code = 0usize;
    for s in states {
        let digit = match s {
            LetterState::Absent(_) => 0,
            LetterState::Present(_, _) => 1,
            LetterState::Correct(_) => 2,
        };
        code = code * 3 + digit;
    }
    code
}

#[cfg(test)]
mod tests {
    use crate::{LetterState, Word, evaluate};

    #[test]
    fn eval_test() {
        let guess: Word = ['t','a','r','e','s'];
        let answer: Word = ['b','u','r','s','t'];
        let result = evaluate(&guess, &answer);
        let expected = [
            LetterState::Present('t', 4),
            LetterState::Absent('a'),
            LetterState::Correct('r'),
            LetterState::Absent('e'),
            LetterState::Present('s', 3)
        ];
        assert_eq!(result, expected)
    }
}