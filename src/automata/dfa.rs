extern crate itertools;

use std::collections::{HashSet,HashMap};
use self::itertools::Itertools;        // fold_results
use std::io;                           // Error
use std::io::{Read,BufReader,BufRead}; // read_to_string
use std::path::Path;
use std::num;                          // ParseIntError
use std::fmt;                          // Formatter, format!, Display, Debug, write!
use std::error;
use std::fs::File;                     // File, open

// TODO "readme.mk"
// TODO documentation
#[derive(Debug)]
pub enum DFAError {
    MissingStartingState,
    MissingFinalStates,
    IncompleteTransition(usize),
    IllformedTransition(usize),
    DuplicatedTransition(usize),
    Io(io::Error),
    Parse(num::ParseIntError,Option<usize>),
}

impl fmt::Display for DFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DFAError::Io(ref err) => write!(f, "IO error: {}", err),
            DFAError::MissingStartingState => write!(f, "The file is empty or only contains white characters."),
            DFAError::MissingFinalStates => write!(f, "The file does not specify the list of final states."),
            DFAError::IncompleteTransition(ref line) => write!(f, "Line {}: missing the src or the dest state.", line),
            DFAError::IllformedTransition(ref line) => write!(f, "Line {}: too much elements.", line),
            DFAError::DuplicatedTransition(ref line) => write!(f, "Line {}: duplicated transition.", line),
            DFAError::Parse(ref err,ref line) => write!(f, "Parse error on line {:?}: {}", line, err),
        }
    }
}

impl error::Error for DFAError {
    fn description(&self) -> &str {
        match *self {
            DFAError::Io(ref err) => err.description(),
            DFAError::MissingStartingState => "The file is empty or only contains white characters.",
            DFAError::MissingFinalStates => "The file does not specify the list of final states.",
            DFAError::IncompleteTransition(_) => "Missing the src or the dest state.",
            DFAError::IllformedTransition(_) => "Too much elements.",
            DFAError::DuplicatedTransition(_) => "Duplicated transition.",
            DFAError::Parse(ref err,_) => err.description(),
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DFAError::Io(ref err) => Some(err),
            DFAError::Parse(ref err,_) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DFAError {
    fn from(err: io::Error) -> DFAError {
        DFAError::Io(err)
    }
}

impl From<num::ParseIntError> for DFAError {
    fn from(err: num::ParseIntError) -> DFAError {
        DFAError::Parse(err,None)
    }
}

#[derive(Debug)]
pub struct DFA {
    transitions : HashMap<(char,usize),usize>,
    start       : usize,
    finals      : HashSet<usize>,
}

#[derive(Debug)]
pub struct DFABuilder {
    transitions : HashMap<(char,usize),usize>,
    start       : usize,
    finals      : HashSet<usize>,
}

pub trait DFABuilding {
    fn add_start(mut self, state: usize) -> Self;
    fn add_final(mut self, state: usize) -> Self;
    fn add_transition(mut self, symb: char, src: usize, dest: usize) -> Self;
    fn finalize(self) -> Result<DFA, DFAError>;
}

impl DFABuilder {
    pub fn new() -> Result<DFABuilder,DFAError> {
        Ok(DFABuilder{transitions: HashMap::new(), start: 0, finals: HashSet::new()})
    }
}

impl DFABuilding for Result<DFABuilder,DFAError> {
    fn add_start(self, state: usize) -> Result<DFABuilder,DFAError> {
        self.and_then(|mut dfa| {
            dfa.start = state;
            Ok(dfa)
        })
    }

    fn add_final(self, state: usize) -> Result<DFABuilder,DFAError> {
        self.and_then(|mut dfa| {
            dfa.finals.insert(state);
            Ok(dfa)
        })
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<DFABuilder,DFAError> {
        self.and_then(|mut dfa| {
            if dfa.transitions.insert((symb,src), dest).is_some() {
                return Err(DFAError::DuplicatedTransition(0));
            }
            Ok(dfa)
        })
    }

    fn finalize(self) -> Result<DFA, DFAError> {
        self.and_then(|dfa| {
            Ok(DFA{transitions: dfa.transitions, start: dfa.start, finals: dfa.finals})
        })
    }
}
pub struct DFAReader;

impl DFAReader {
    fn parse_dfa_error(contents: &str, line: usize) -> Result<usize, DFAError> {
            contents.parse::<usize>()
                    .map_err(|e| DFAError::Parse(e,Some(line)))
    }

    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<DFA, DFAError> {
        let file = try!(File::open(file_path));
        let file = BufReader::new(file);
        DFAReader::new_from_lines(&mut file.lines())
    }

    fn new_from_lines(lines : &mut Iterator<Item=io::Result<String>>) -> Result<DFA, DFAError> {
        let mut dfa = DFABuilder::new().finalize().unwrap();
        //let mut dfa = DFABuilder::new();
        let mut lines = lines
            .map(|line| {
                line.and_then(|contents| Ok(contents.split('#').nth(0).unwrap().trim().to_owned()))
            })
            .enumerate().map(|(nline,line)| (nline+1,line))
            .filter(|&(_,ref line)| {
                // Mandatory otherwise unwrap will take the ownership of the String
                let line = line.as_ref();
                line.is_err() || !line.unwrap().is_empty()
            });
        // Starting state
        let (nline,line) = try!(lines.next().ok_or(DFAError::MissingStartingState));
        let line = try!(line);
        dfa.start = try!(DFAReader::parse_dfa_error(&line,nline));
        //let start = try!(DFAReader::parse_dfa_error(&line,nline));
        //try!(dfa.add_start(start).map_err(|e| );
        // Final states
        let (nline,line) = try!(lines.next().ok_or(DFAError::MissingFinalStates));
        let line = try!(line);
        dfa.finals = try!(line
            .split_whitespace()
            .map(|token| DFAReader::parse_dfa_error(token,nline))
            .fold_results(HashSet::new(), |mut acc, elt| {
                acc.insert(elt);
                acc
            }));
        //try!(line
            //.split_whitespace()
            //.map(|token| DFAReader::parse_dfa_error(token,nline))
            //.fold_results(dfa, |mut acc, elt| acc.add_final(elt)));
        //try!(dfa.map_err(|e| ));
        for (nline,line) in lines {
            let line = try!(line);
            let mut tokens = line.split_whitespace();
            // can't fail because lines iterates over the non-empty line
            let mut symbs = tokens.next().unwrap().chars();
            let symb = symbs.nth(0).unwrap();
            if symbs.next().is_some() {
                return Err(DFAError::IllformedTransition(nline));
            }
            let src = try!(tokens
                .next()
                .ok_or(DFAError::IncompleteTransition(nline))
                .and_then(|contents| DFAReader::parse_dfa_error(contents,nline)));
            let dest = try!(tokens
                .next()
                .ok_or(DFAError::IncompleteTransition(nline))
                .and_then(|contents| DFAReader::parse_dfa_error(contents,nline)));
            if tokens.next().is_some() {
                return Err(DFAError::IllformedTransition(nline));
            }
            if dfa.transitions.insert((symb,src), dest).is_some() {
                return Err(DFAError::DuplicatedTransition(nline));
            }
            //try!(dfa.add_transition(symb,src,dest).map_err(|e| ));
        }
        Ok(dfa)
    }

    pub fn new_from_string(file: &str) -> Result<DFA, DFAError> {
        DFAReader::new_from_lines(&mut file.lines().map(|line| Ok(line.to_string())))
    }
}

impl DFA {
    // TODO return the position of the first match
    //      maybe create an another function to do that
    pub fn run(&self, input: &str) -> bool {
        let f = input
            .chars()
            .fold(Some(self.start), |state,c| {
                match state {
                    Some(n) => self.transitions.get(&(c,n)).map(|v| *v),
                    None => None,
                }
            });
        match f {
            Some(n) => self.finals.contains(&n),
            None => false
        }
    }
}

impl fmt::Display for DFA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "START: {}", self.start));
        try!(writeln!(f, "FINALS:"));
        for fi in self.finals.iter() {
            try!(writeln!(f,"  {}", fi));
        }
        try!(writeln!(f, "TRANSITIONS:"));
        for (tr,d) in self.transitions.iter() {
            let (c,s) = *tr;
            try!(writeln!(f, "  ({},{}) => {}", c, s, d));
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa() {
        let model =
            "0\n\
             3\n\
             a 0 1\n\
             c 0 3\n\
             b 1 2\n\
             a 2 1\n\
             c 2 3";
        let dfa = DFAReader::new_from_string(&model).unwrap();
        let samples =
            vec![("ababac", false),
                 ("ababc", true),
                 ("", false),
                 ("abc", true),
                 ("c", true),
                 ("ac", false),
                 ("ababababababababababababababababababababc", true),];

        for (input,expected_result) in samples {
            assert!(dfa.run(input) == expected_result, "input false for: \"{}\"", input);
        }
    }

    #[test]
    fn test_empty_file() {
        let model =
            "";
        match DFAReader::new_from_string(model) {
            Err(DFAError::MissingStartingState) => assert!(true),
            _ => assert!(false, "Missing state expected."),
        }
    }

    #[test]
    fn test_start_not_a_number() {
        let model =
            "a";
        match DFAReader::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 1),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_many_starts() {
        let model =
            "0 1\n\
             3\n\
             a 0 1\n\
             c 0 3\n\
             b 1 2\n\
             a 2 1\n\
             c 2 3";
        match DFAReader::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 1),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        match DFAReader::new_from_string(model) {
            Err(DFAError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "Missing final states expected."),
        }
    }

    #[test]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        match DFAReader::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 2),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_no_transistions() {
        let model =
            "0\n\
             3";
        let _dfa = DFAReader::new_from_string(&model).unwrap();
    }

    #[test]
    fn test_transitions_with_at_least_four_elements() {
        let model =
            "0\n\
             3\n\
             a 0 1 8";
        match DFAReader::new_from_string(model) {
            Err(DFAError::IllformedTransition(line)) => assert!(line == 3),
            _ => assert!(false, "IllformedTransition expected."),
        }
    }

    #[test]
    #[should_panic]
    fn test_transitions_start_with_at_least_two_chars() {
        let model =
            "0\n\
             3\n\
             ab 2 3";
        let _dfa = DFAReader::new_from_string(&model).unwrap();
    }

    #[test]
    fn test_transitions_with_src_not_a_number() {
        let model =
            "0\n\
             3\n\
             c b 3";
        match DFAReader::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 3),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_transitions_with_dest_not_a_number() {
        let model =
            "0\n\
             3\n\
             c 2 b";
        match DFAReader::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 3),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_duplicated_transition() {
        let model =
            "0\n\
             3\n\
             c 2 3\n\
             c 2 4";
        match DFAReader::new_from_string(model) {
            Err(DFAError::DuplicatedTransition(line)) => assert!(line == 4),
            _ => assert!(false, "DuplicatedTransition expected."),
        }
    }

    #[test]
    fn test_read_from_fake_file() {
        let file = "fake.txt";
        match DFAReader::new_from_file(file) {
            Err(DFAError::Io(_)) => assert!(true),
            _ => assert!(false, "Io::Error expected."),
        }
    }
}
