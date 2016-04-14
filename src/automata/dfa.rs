extern crate itertools;

use std::collections::{HashSet,HashMap};
use self::itertools::Itertools;
use std::fmt;
use std::error;
use std::num;

// TODO add specific errors
// TODO add the line in the error
// TODO remove the Option after the add of specific erros
#[derive(Debug)]
pub enum DFAError {
    Io(String,Option<usize>),
    Parse(num::ParseIntError,Option<usize>),
}

impl fmt::Display for DFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DFAError::Io(ref err,ref line) => write!(f, "IO error on line {:?}: {}", line, err),
            DFAError::Parse(ref err,ref line) => write!(f, "Parse error on line {:?}: {}", line, err),
        }
    }
}
impl error::Error for DFAError {
    fn description(&self) -> &str {
        match *self {
            DFAError::Io(ref err,ref _line) => err,
            DFAError::Parse(ref err,ref _line) => err.description(),
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DFAError::Io(ref _err,ref _line) => Some(self),
            DFAError::Parse(ref err,ref _line) => Some(err),
        }
    }
}

impl From<String> for DFAError {
    fn from(err: String) -> DFAError {
        DFAError::Io(err,None)
    }
}

impl From<num::ParseIntError> for DFAError {
    fn from(err: num::ParseIntError) -> DFAError {
        DFAError::Parse(err,None)
    }
}

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

    // TODO read from a "File"
    // TODO use specific erros
    // TODO remove string inside erro
    // TODO remove string for error desciption
    pub fn new_from_file(file: &str) -> Result<DFA, DFAError> {
        let mut dfa = DFA::new();
        // lines iterates over the non-empty lines.
        let mut lines = file
            .lines()
            .enumerate()
            .map(|(nline,line)| (nline+1,line.trim()))
            .filter(|&(_,line)| !line.is_empty());
        dfa.start = try!(lines
            .next()
            .ok_or(DFAError::Io("The file does not contain the starting state.".to_owned(),None))
            .and_then(|(nline,contents)| DFA::parse_dfa_error(contents,nline)));
        dfa.finals = try!(lines
            .next()
            .ok_or(DFAError::Io("The file does not contain the list of finals state.".to_owned(),None))
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
            let symb = try!(tokens
                .next()
                .ok_or(DFAError::Io("Should not happen.".to_owned(),Some(nline)))
                .and_then(|contents| {
                    contents.chars()
                            .nth(0)
                            .ok_or(DFAError::Io("Should not happen.".to_owned(),Some(nline)))
                }));
            let src = try!(tokens
                .next()
                .ok_or(DFAError::Io("The transition line does not contain the src state.".to_owned(),Some(nline)))
                .and_then(|contents| DFA::parse_dfa_error(contents,nline)));
            let dest = try!(tokens
                .next()
                .ok_or(DFAError::Io("The transition line does not contain the dest state.".to_owned(),Some(nline)))
                .and_then(|contents| DFA::parse_dfa_error(contents,nline)));
            if tokens.next().is_some() {
                return Err(DFAError::Io("Too much elements on the transition line.".to_owned(),Some(nline)));
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
        let dfa = DFA::new_from_file(&model).unwrap();
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
    #[should_panic]
    fn test_empty_file() {
        let model =
            "";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_start_not_a_number() {
        let model =
            "a";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_many_starts() {
        let model =
            "0 1\n\
             3\n\
             a 0 1\n\
             c 0 3\n\
             b 1 2\n\
             a 2 1\n\
             c 2 3";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    fn test_no_transistions() {
        let model =
            "0\n\
             3";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_transitions_with_at_least_four_elements() {
        let model =
            "0 1\n\
             3\n\
             a 0 1 8";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_transitions_start_with_at_least_two_chars() {
        let model =
            "0 1\n\
             3\n\
             ab 2 3";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_transitions_with_src_not_a_number() {
        let model =
            "0 1\n\
             3\n\
             c b 3";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_transitions_with_dest_not_a_number() {
        let model =
            "0 1\n\
             3\n\
             c 2 b";
        let _dfa = DFA::new_from_file(&model).unwrap();
    }
}
