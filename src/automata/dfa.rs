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
    DuplicatedTransition(char,usize),
    MissingFinalStates,
}

#[derive(Debug)]
pub enum DFAReaderError {
    MissingStartingState,
    MissingFinalStates,
    IncompleteTransition(usize),
    IllformedTransition(usize),
    DFA(DFAError,usize),
    Io(io::Error),
    Parse(num::ParseIntError,usize),
}


impl fmt::Display for DFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DFAError::DuplicatedTransition(symb,state) => write!(f, "Duplicated transition ('{}',{}).", symb, state),
            DFAError::MissingFinalStates => write!(f, "Missing final states."),
        }
    }
}

impl error::Error for DFAError {
    fn description(&self) -> &str {
        match *self {
            DFAError::DuplicatedTransition(_,_) => "Duplicated transition.", 
            DFAError::MissingFinalStates => "Missing final states.",
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for DFAReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DFAReaderError::Io(ref err) => write!(f, "IO error: {}", err),
            DFAReaderError::MissingStartingState => write!(f, "The file is empty or only contains white characters."),
            DFAReaderError::MissingFinalStates => write!(f, "The file does not specify the list of final states."),
            DFAReaderError::IncompleteTransition(ref line) => write!(f, "Line {}: missing the src or the dest state.", line),
            DFAReaderError::IllformedTransition(ref line) => write!(f, "Line {}: too much elements.", line),
            DFAReaderError::DFA(ref err,ref line) => write!(f, "Line {}: DFAError {}", line, err),
            DFAReaderError::Parse(ref err,ref line) => write!(f, "Line {}: parse error {}", line, err),
        }
    }
}

impl error::Error for DFAReaderError {
    fn description(&self) -> &str {
        match *self {
            DFAReaderError::Io(ref err) => err.description(),
            DFAReaderError::MissingStartingState => "The file is empty or only contains white characters.",
            DFAReaderError::MissingFinalStates => "The file does not specify the list of final states.",
            DFAReaderError::IncompleteTransition(_) => "Missing the src or the dest state.",
            DFAReaderError::IllformedTransition(_) => "Too much elements.",
            DFAReaderError::DFA(ref err,_) => err.description(),
            DFAReaderError::Parse(ref err,_) => err.description(),
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DFAReaderError::Io(ref err) => Some(err),
            DFAReaderError::Parse(ref err,_) => Some(err),
            DFAReaderError::DFA(ref err,_) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DFAReaderError {
    fn from(err: io::Error) -> DFAReaderError {
        DFAReaderError::Io(err)
    }
}

impl From<num::ParseIntError> for DFAReaderError {
    fn from(err: num::ParseIntError) -> DFAReaderError {
        DFAReaderError::Parse(err,0)
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
                return Err(DFAError::DuplicatedTransition(symb,src));
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
    fn parse_dfa_error(contents: &str, line: usize) -> Result<usize, DFAReaderError> {
            contents.parse::<usize>()
                    .map_err(|e| DFAReaderError::Parse(e,line))
    }

    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<DFA, DFAReaderError> {
        let file = try!(File::open(file_path));
        let file = BufReader::new(file);
        DFAReader::new_from_lines(&mut file.lines())
    }

    fn new_from_lines(lines : &mut Iterator<Item=io::Result<String>>) -> Result<DFA, DFAReaderError> {
        let mut dfa = DFABuilder::new();
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
        let (nline,line) = try!(lines.next().ok_or(DFAReaderError::MissingStartingState));
        let line = try!(line);
        let start = try!(DFAReader::parse_dfa_error(&line,nline));
        dfa = dfa.add_start(start);
        if let Err(e) = dfa {
            return Err(DFAReaderError::DFA(e,nline));
        }
        // Final states
        let (nline,line) = try!(lines.next().ok_or(DFAReaderError::MissingFinalStates));
        let line = try!(line);
        dfa = try!(line
            .split_whitespace()
            .map(|token| DFAReader::parse_dfa_error(token,nline))
            .fold_results(dfa, |acc, elt| acc.add_final(elt)));
        if let Err(e) = dfa {
            return Err(DFAReaderError::DFA(e,nline));
        }
        // Transitions
        for (nline,line) in lines {
            let line = try!(line);
            let mut tokens = line.split_whitespace();
            // can't fail because lines iterates over the non-empty line
            let mut symbs = tokens.next().unwrap().chars();
            let symb = symbs.nth(0).unwrap();
            if symbs.next().is_some() {
                return Err(DFAReaderError::IllformedTransition(nline));
            }
            let src = try!(tokens
                .next()
                .ok_or(DFAReaderError::IncompleteTransition(nline))
                .and_then(|contents| DFAReader::parse_dfa_error(contents,nline)));
            let dest = try!(tokens
                .next()
                .ok_or(DFAReaderError::IncompleteTransition(nline))
                .and_then(|contents| DFAReader::parse_dfa_error(contents,nline)));
            if tokens.next().is_some() {
                return Err(DFAReaderError::IllformedTransition(nline));
            }
            dfa = dfa.add_transition(symb,src,dest);
            if let Err(e) = dfa {
                return Err(DFAReaderError::DFA(e,nline));
            }
        }
        dfa.finalize().map_err(|e| DFAReaderError::DFA(e,nline))
    }

    pub fn new_from_string(file: &str) -> Result<DFA, DFAReaderError> {
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
    fn test_dfa_builder() {
        let _dfa = DFABuilder::new()
            .add_start(0)
            .add_final(3)
            .add_transition('a', 0, 1)
            .add_transition('c', 0, 3)
            .add_transition('b', 1, 2)
            .add_transition('a', 2, 1)
            .add_transition('c', 2, 3)
            .unwrap();
    }

    #[test]
    fn test_dfa_builder_duplicated_transition() {
        let dfa = DFABuilder::new()
            .add_start(0)
            .add_final(3)
            .add_transition('a', 0, 1)
            .add_transition('c', 0, 3)
            .add_transition('b', 1, 2)
            .add_transition('a', 2, 1)
            .add_transition('c', 2, 3)
            .add_transition('a', 0, 2);
        match dfa {
            Err(DFAError::DuplicatedTransition(sy,sr)) => assert!((sy,sr) == ('a',0)),
            _ => assert!(false, "DuplicatedTransition expected."),
        }
    }

    #[test]
    fn test_empty_file() {
        let model =
            "";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::MissingStartingState) => assert!(true),
            _ => assert!(false, "Missing state expected."),
        }
    }

    #[test]
    fn test_start_not_a_number() {
        let model =
            "a";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 1),
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
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 1),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "Missing final states expected."),
        }
    }

    #[test]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 2),
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
            Err(DFAReaderError::IllformedTransition(line)) => assert!(line == 3),
            _ => assert!(false, "IllformedTransition expected."),
        }
    }

    #[test]
    fn test_transitions_start_with_at_least_two_chars() {
        let model =
            "0\n\
             3\n\
             ab 2 3";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::IllformedTransition(line)) => assert!(line == 3),
            _ => assert!(false, "IllformedTransition expected."),
        }
    }

    #[test]
    fn test_transitions_with_src_not_a_number() {
        let model =
            "0\n\
             3\n\
             c b 3";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 3),
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
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 3),
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
            Err(DFAReaderError::DFA(_,line)) => assert!(line == 4),
            _ => assert!(false, "DuplicatedTransition expected."),
        }
    }

    #[test]
    fn test_read_from_fake_file() {
        let file = "fake.txt";
        match DFAReader::new_from_file(file) {
            Err(DFAReaderError::Io(_)) => assert!(true),
            _ => assert!(false, "Io::Error expected."),
        }
    }
}
