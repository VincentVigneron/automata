extern crate itertools;

use std::collections::{HashSet,HashMap};
use self::itertools::Itertools;
use std::fmt;
use std::error;
use std::io;
use std::num;
use std::fs::File;
use std::path::Path;
use std::io::Read;

// TODO "readm.mk"
// TODO documentation
// TODO read from an iterator
// TODO error for duplicated transitions
#[derive(Debug)]
pub enum DFAError {
    MissingStartingState,
    MissingFinalStates,
    IncompleteTransition(usize),
    IllformedTransition(usize),
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
            DFAError::IllformedTransition(_) => "Too much elements",
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

impl DFA {
    pub fn new() -> DFA {
        DFA{transitions: HashMap::new(), start: 0, finals: HashSet::new()}
    }

    fn parse_dfa_error(contents: &str, line: usize) -> Result<usize, DFAError> {
            contents.parse::<usize>()
                    .map_err(|e| DFAError::Parse(e,Some(line)))
    }

    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<DFA, DFAError> {
        let mut file = try!(File::open(file_path));
        let mut contents = String::new();
        try!(file.read_to_string(&mut contents));
        DFA::new_from_string(&*contents)
    }

    // TODO test if the tranisiton start with two symbols instead of one
    pub fn new_from_string(file: &str) -> Result<DFA, DFAError> {
        let mut dfa = DFA::new();
        // lines iterates over the non-empty lines.
        let mut lines = file
            .lines()
            .enumerate()
            .map(|(nline,line)| (nline+1,line.trim()))
            .filter(|&(_,line)| !line.is_empty());
        dfa.start = try!(lines
            .next()
            .ok_or(DFAError::MissingStartingState)
            .and_then(|(nline,contents)| DFA::parse_dfa_error(contents,nline)));
        dfa.finals = try!(lines
            .next()
            .ok_or(DFAError::MissingFinalStates)
            .and_then(|(nline,contents)| Ok((nline,contents.split_whitespace())))
            // need to move the closure inside the make to create a copy of nline for
            // th closure stack frame, otherwise the compiler can't guarantee that the
            // lifetime of the closure won't exceed the lifetime of nline.
            .and_then(|(nline,contents)| Ok((nline,contents.map(move |token| DFA::parse_dfa_error(token,nline)))))
            .and_then(|(_, mut contents)| {
                contents.fold_results(HashSet::new(),|mut acc,elt| {
                    acc.insert(elt);
                    acc
                })
            }));
        for (nline,line) in lines {
            let mut tokens = line.split_whitespace();
            // can't fail because lines iterates over the non-empty line
            let symb = tokens.next().unwrap().chars().nth(0).unwrap();
            let src = try!(tokens
                .next()
                .ok_or(DFAError::IncompleteTransition(nline))
                .and_then(|contents| DFA::parse_dfa_error(contents,nline)));
            let dest = try!(tokens
                .next()
                .ok_or(DFAError::IncompleteTransition(nline))
                .and_then(|contents| DFA::parse_dfa_error(contents,nline)));
            if tokens.next().is_some() {
                return Err(DFAError::IllformedTransition(nline));
            }
            dfa.transitions.insert((symb,src), dest);
        }
        Ok(dfa)
    }

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

// TODO add specific error analysis
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
        let dfa = DFA::new_from_string(&model).unwrap();
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
        match DFA::new_from_string(model) {
            Err(DFAError::MissingStartingState) => assert!(true),
            _ => assert!(false, "Missing state expected."),
        }
    }

    #[test]
    fn test_start_not_a_number() {
        let model =
            "a";
        match DFA::new_from_string(model) {
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
        match DFA::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 1),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        match DFA::new_from_string(model) {
            Err(DFAError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "Missing final states expected."),
        }
    }

    #[test]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        match DFA::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 2),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_no_transistions() {
        let model =
            "0\n\
             3";
        let _dfa = DFA::new_from_string(&model).unwrap();
    }

    #[test]
    fn test_transitions_with_at_least_four_elements() {
        let model =
            "0\n\
             3\n\
             a 0 1 8";
        match DFA::new_from_string(model) {
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
        let _dfa = DFA::new_from_string(&model).unwrap();
    }

    #[test]
    fn test_transitions_with_src_not_a_number() {
        let model =
            "0\n\
             3\n\
             c b 3";
        match DFA::new_from_string(model) {
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
        match DFA::new_from_string(model) {
            Err(DFAError::Parse(_,line)) => assert!(line.unwrap() == 3),
            _ => assert!(false, "Parsing error."),
        }
    }

    #[test]
    fn test_read_from_fake_file() {
        let file = "fake.txt";
        match DFA::new_from_file(file) {
            Err(DFAError::Io(_)) => assert!(true),
            _ => assert!(false, "Io::Error expected."),
        }
    }
}
