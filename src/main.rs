use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use std::collections::{HashMap, VecDeque, HashSet};
use std::fmt::{Debug, Formatter, Display};
use std::fmt;

const ALPHABET: &str = "ABCDEFGIJKLMNOPQRSTUVWXYZ";

struct GridPos {
    row: usize,
    col: usize,
}

// this is reused everywhere :  )
#[derive(Clone, Debug)]
struct InvalidInput(String);

impl FromStr for GridPos {
    type Err = InvalidInput;

    fn from_str(input: &str) -> Result<GridPos, InvalidInput> {
        let col = match input.chars().nth(0).and_then(|c| ALPHABET.find(c)) {
            Some(col) => col,
            None => return Err(InvalidInput(input.to_owned())),
        };

        let row = match input[1..].parse::<usize>() {
            Ok(row) => row - 1,
            Err(_) => return Err(InvalidInput(input.to_owned())),
        };

        Ok(GridPos { row, col })
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct CandidateId(usize);

impl Debug for CandidateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for CandidateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            1 => "Medieval Fantasy",
            2 => "Alternate Universe",
            3 => "Steampunk",
            4 => "Vestiges",
            5 => "Invasion",
            _ => "Unknown",
        })
    }
}

impl FromStr for CandidateId {
    type Err = InvalidInput;

    fn from_str(input: &str) -> Result<Self, InvalidInput> {
        input
            .rmatches(char::is_numeric)
            .next()
            .and_then(|c| c.parse::<usize>().ok())
            .map(CandidateId)
            .ok_or(InvalidInput(input.to_owned()))
    }
}

struct Voter {
    choices: VecDeque<CandidateId>,
}

impl Voter {
    fn try_parse_from_row<'a>(record: impl Iterator<Item = &'a str>) -> Result<Voter, InvalidInput> {
        Ok(Voter { choices: record.map(CandidateId::from_str).collect::<Result<_, _>>()? })
    }
}

fn main() {
    let start: GridPos = "R3".parse().unwrap();
    let end: GridPos = "V18".parse().unwrap();

    let file_reader = BufReader::new(File::open("responses.csv").unwrap());
    let mut csv_reader = csv::Reader::from_reader(file_reader);

    println!("==== Counting ballots ====");
    let mut voters: Vec<Voter> = csv_reader
        .records()
        .skip(start.row - 1)
        .take(end.row - start.row + 1)
        .map(Result::unwrap)
        .inspect(|row| {
            print!("Counting ballot: ");
            for v in row.iter().skip(start.col + 1).take(end.col - start.col + 1) {
                print!("{}; ", v);
            }
            println!();
        })
        .map(|row| Voter::try_parse_from_row(row.iter().skip(start.col + 1).take(end.col - start.col + 1)))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    println!("\nIn total, {} ballots were counted.", voters.len());

    let mut eliminated: HashSet<CandidateId> = HashSet::new();
    let total_votes = voters.len();

    // Tally first choice votes
    let mut round = 0;
    loop {
        let mut tally: HashMap<CandidateId, usize> = HashMap::new();
        round += 1;
        for voter in &voters {
            *tally.entry(voter.choices[0]).or_default() += 1;
        }

        println!("==== Round {} ====", round);
        println!("The current tally is: {:#?}", tally);

        let top_candidate = tally.iter().max_by_key(|(_candidate, votes)| **votes).unwrap();
        if *top_candidate.1 as f32 > (total_votes as f32 / 2.0) {
            println!(
                "The winner is {}, with {}% of the final vote, or {} out of {} votes!",
                top_candidate.0,
                (*top_candidate.1 as f32 / total_votes as f32) * 100.0,
                top_candidate.1,
                total_votes,
            );
            return;
        } else {
            println!(
                "The current leader is {}, with {}% of the final vote, or {} out of {} votes.",
                top_candidate.0,
                (*top_candidate.1 as f32 / total_votes as f32) * 100.0,
                top_candidate.1,
                total_votes,
            );
        }

        let eliminate = tally.iter().min_by_key(|(_candidate, votes)| **votes).unwrap();
        println!(
            "Eliminating {}, which had {}% of the current vote, or {} out of {} votes.",
            eliminate.0,
            (*eliminate.1 as f32 / total_votes as f32) * 100.0,
            eliminate.1,
            total_votes,
        );

        eliminated.insert(*eliminate.0);

        for voter in voters.iter_mut() {
            while eliminated.contains(voter.choices.front().unwrap()) {
                voter.choices.pop_front();
            }
        }
    }
}
