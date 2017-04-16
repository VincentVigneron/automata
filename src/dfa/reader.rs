// Copyright 2016 Vincent Vigneron. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at.your option.
// This file may not be copied, modified, or distributed
// except according to those terms.

extern crate itertools;

use std::io;                           // Error
use std::io::{BufReader,BufRead}; // read_to_string
use std::path::Path;
use std::num;                          // ParseIntError
use std::fmt;                          // Formatter, format!, Display, Debug, write!
use std::error;
use std::fs::File;                     // File, open
use std::result;
use self::itertools::Itertools;        // fold_results

use dfa::core::{DFA,DFABuilder,DFAError,DFABuilding};

/// Type `DFAReaderError` describes the list of errors that can occur during
/// the parsing of a DFA file.
#[derive(Debug)]
pub enum DFAReaderError {
    /// Error `MissingStartingState` means the file does not contains the starting state.
    MissingStartingState,
    /// Error `MissingFinalStates` means the file does not contains the list of final states.
    MissingFinalStates,
    /// Error `IncompleteTransition` means the transition on the specified line does not contain
    /// one of these elements: symbol, source state, destination state.
    IncompleteTransition(usize),
    /// Error `IllformedTransition` means the transition contains to much elements or that
    /// the symbole is composed with modre than two characters.
    IllformedTransition(usize),
    /// Error `DFA` encapsules the error specific to the DFA building process (no final
    /// states,...).
    DFA(DFAError,usize),
    /// Error `Io` is relative to the input errors (the file does not exist, the file can not be
    /// read,...à.
    Io(io::Error),
    /// Error `Parse` is relative to the parsing errors (a state is an intger).
    Parse(num::ParseIntError,usize),
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

/// Alias for result::Result<T,DFAReaderError>.
pub type Result<T> = result::Result<T,DFAReaderError>;

/// Struct `DFAReader` is an empty structure that builds a `DFA` from a file
/// or from a `&str`.
pub struct DFAReader;

impl DFAReader {
    fn parse_dfa_error(contents: &str, line: usize) -> Result<usize> {
            contents.parse::<usize>()
                    .map_err(|e| DFAReaderError::Parse(e,line))
    }

    /// Reads a DFA from a file.
    ///
    /// # Description
    ///
    /// * `file_path` - The path to the file that contains the DFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::dfa::reader::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     let dfa = DFAReader::new_from_file("dfa.txt");
    ///     match dfa {
    ///         Ok(dfa) => {
    ///            // Do stuff with the dfa
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<DFA> {
        let file = try!(File::open(file_path));
        let file = BufReader::new(file);
        DFAReader::new_from_lines(&mut file.lines())
    }

    fn read_start(dfa: DFABuilder, lines : &mut Iterator<Item=(usize,io::Result<String>)>) -> Result<DFABuilder> {
        let (nline,line) = try!(lines.next().ok_or(DFAReaderError::MissingStartingState));
        let line = try!(line);
        let start = try!(DFAReader::parse_dfa_error(&line,nline));
        let dfa = dfa.add_start(start);
        match dfa {
            Ok(dfa) => Ok(dfa),
            Err(e) => Err(DFAReaderError::DFA(e,nline)),
        }
    }

    fn read_finals(dfa: DFABuilder, lines : &mut Iterator<Item=(usize,io::Result<String>)>) -> Result<DFABuilder> {
        let (nline,line) = try!(lines.next().ok_or(DFAReaderError::MissingFinalStates));
        let line = try!(line);
        let dfa = try!(try!(line
            .split_whitespace()
            .map(|token| DFAReader::parse_dfa_error(token,nline))
            .fold_results(Ok(dfa), |acc, elt| acc.add_final(elt)))
            .map_err(|e| DFAReaderError::DFA(e,nline)));
        Ok(dfa)
    }

    fn read_transition(dfa: DFABuilder, line : (usize,io::Result<String>))-> Result<DFABuilder> {
        let (nline,line) = line;
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
        let dfa = try!(dfa.add_transition(symb,src,dest).map_err(|e| DFAReaderError::DFA(e,nline)));;
        Ok(dfa)
    }

    fn new_from_lines(lines : &mut Iterator<Item=io::Result<String>>) -> Result<DFA> {
        let mut dfa = try!(DFABuilder::new().map_err(|e| DFAReaderError::DFA(e,0)));
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
        dfa = try!(DFAReader::read_start(dfa, &mut lines));
        dfa = try!(DFAReader::read_finals(dfa, &mut lines));
        for line in lines {
            dfa = try!(DFAReader::read_transition(dfa, line));
        }
        dfa.finalize().map_err(|e| DFAReaderError::DFA(e,0))
    }

    /// Reads a DFA from a `&str`.
    ///
    /// # Description
    ///
    /// * `dfa` - The string representation of the DFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::dfa::reader::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     // (abc)*
    ///     let dfa =
    ///         "0\n\
    ///          0\n\
    ///          a 0 1\n\
    ///          b 1 2\n\
    ///          c 2 0";
    ///     let dfa = DFAReader::new_from_string(dfa);
    ///     match dfa {
    ///         Ok(dfa) => {
    ///            // Do stuff with the dfa
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn new_from_string(dfa: &str) -> Result<DFA> {
        DFAReader::new_from_lines(&mut dfa.lines().map(|line| Ok(line.to_string())))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_file() {
        let model =
            "";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::MissingStartingState) => assert!(true),
            _ => assert!(false, "MissingStartingState expected."),
        }
    }

    #[test]
    fn test_start_not_a_number() {
        let model =
            "a";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 1),
            _ => assert!(false, "Parse expected."),
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
            _ => assert!(false, "Parse expected."),
        }
    }

    #[test]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "MissingFinalStates expected."),
        }
    }

    #[test]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        match DFAReader::new_from_string(model) {
            Err(DFAReaderError::Parse(_,line)) => assert!(line == 2),
            _ => assert!(false, "Parse expected."),
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
            _ => assert!(false, "Parse expected."),
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
            _ => assert!(false, "Parse expected."),
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
