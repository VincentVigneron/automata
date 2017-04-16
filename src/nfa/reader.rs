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

use nfa::core::{NFA,NFABuilder,NFAError,NFABuilding};

/// Type `NFAReaderError` describes the list of errors that can occur during
/// the parsing of a NFA file.
#[derive(Debug)]
pub enum NFAReaderError {
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
    /// Error `NFA` encapsules the error specific to the NFA building process (no final
    /// states,...).
    NFA(NFAError,usize),
    /// Error `Io` is relative to the input errors (the file does not exist, the file can not be
    /// read,...à.
    Io(io::Error),
    /// Error `Parse` is relative to the parsing errors (a state is an intger).
    Parse(num::ParseIntError,usize),
}

impl fmt::Display for NFAReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NFAReaderError::Io(ref err) => write!(f, "IO error: {}", err),
            NFAReaderError::MissingStartingState => write!(f, "The file is empty or only contains white characters."),
            NFAReaderError::MissingFinalStates => write!(f, "The file does not specify the list of final states."),
            NFAReaderError::IncompleteTransition(ref line) => write!(f, "Line {}: missing the src or the dest state.", line),
            NFAReaderError::IllformedTransition(ref line) => write!(f, "Line {}: too much elements.", line),
            NFAReaderError::NFA(ref err,ref line) => write!(f, "Line {}: NFAError {}", line, err),
            NFAReaderError::Parse(ref err,ref line) => write!(f, "Line {}: parse error {}", line, err),
        }
    }
}

impl error::Error for NFAReaderError {
    fn description(&self) -> &str {
        match *self {
            NFAReaderError::Io(ref err) => err.description(),
            NFAReaderError::MissingStartingState => "The file is empty or only contains white characters.",
            NFAReaderError::MissingFinalStates => "The file does not specify the list of final states.",
            NFAReaderError::IncompleteTransition(_) => "Missing the src or the dest state.",
            NFAReaderError::IllformedTransition(_) => "Too much elements.",
            NFAReaderError::NFA(ref err,_) => err.description(),
            NFAReaderError::Parse(ref err,_) => err.description(),
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        match *self {
            NFAReaderError::Io(ref err) => Some(err),
            NFAReaderError::Parse(ref err,_) => Some(err),
            NFAReaderError::NFA(ref err,_) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for NFAReaderError {
    fn from(err: io::Error) -> NFAReaderError {
        NFAReaderError::Io(err)
    }
}

impl From<num::ParseIntError> for NFAReaderError {
    fn from(err: num::ParseIntError) -> NFAReaderError {
        NFAReaderError::Parse(err,0)
    }
}

/// Alias for result::Result<T,NFAReaderError>.
pub type Result<T> = result::Result<T,NFAReaderError>;

/// Struct `NFAReader` is an empty structure that builds a `NFA` from a file
/// or from a `&str`.
pub struct NFAReader;

impl NFAReader {
    fn parse_nfa_error(contents: &str, line: usize) -> Result<usize> {
            contents.parse::<usize>()
                    .map_err(|e| NFAReaderError::Parse(e,line))
    }

    /// Reads a NFA from a file.
    ///
    /// # Description
    ///
    /// * `file_path` - The path to the file that contains the NFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::nfa::reader::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     let nfa = NFAReader::new_from_file("nfa.txt");
    ///     match nfa {
    ///         Ok(nfa) => {
    ///            // Do stuff with the nfa
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<NFA> {
        let file = try!(File::open(file_path));
        let file = BufReader::new(file);
        NFAReader::new_from_lines(&mut file.lines())
    }

    fn read_start(nfa: NFABuilder, lines : &mut Iterator<Item=(usize,io::Result<String>)>) -> Result<NFABuilder> {
        let (nline,line) = try!(lines.next().ok_or(NFAReaderError::MissingStartingState));
        let line = try!(line);
        let start = try!(NFAReader::parse_nfa_error(&line,nline));
        let nfa = nfa.add_start(start);
        match nfa {
            Ok(nfa) => Ok(nfa),
            Err(e) => Err(NFAReaderError::NFA(e,nline)),
        }
    }

    fn read_finals(nfa: NFABuilder, lines : &mut Iterator<Item=(usize,io::Result<String>)>) -> Result<NFABuilder> {
        let (nline,line) = try!(lines.next().ok_or(NFAReaderError::MissingFinalStates));
        let line = try!(line);
        let nfa = try!(try!(line
            .split_whitespace()
            .map(|token| NFAReader::parse_nfa_error(token,nline))
            .fold_results(Ok(nfa), |acc, elt| acc.add_final(elt)))
            .map_err(|e| NFAReaderError::NFA(e,nline)));
        Ok(nfa)
    }

    fn read_transition(nfa: NFABuilder, line : (usize,io::Result<String>))-> Result<NFABuilder> {
        let (nline,line) = line;
        let line = try!(line);
        let mut tokens = line.split_whitespace();
        // can't fail because lines iterates over the non-empty line
        let mut symbs = tokens.next().unwrap().chars();
        let symb = symbs.nth(0).unwrap();
        if symbs.next().is_some() {
            return Err(NFAReaderError::IllformedTransition(nline));
        }
        let src = try!(tokens
            .next()
            .ok_or(NFAReaderError::IncompleteTransition(nline))
            .and_then(|contents| NFAReader::parse_nfa_error(contents,nline)));
        let dest = try!(tokens
            .next()
            .ok_or(NFAReaderError::IncompleteTransition(nline))
            .and_then(|contents| NFAReader::parse_nfa_error(contents,nline)));
        if tokens.next().is_some() {
            return Err(NFAReaderError::IllformedTransition(nline));
        }
        let nfa = try!(nfa.add_transition(symb,src,dest).map_err(|e| NFAReaderError::NFA(e,nline)));;
        Ok(nfa)
    }

    fn new_from_lines(lines : &mut Iterator<Item=io::Result<String>>) -> Result<NFA> {
        let mut nfa = try!(NFABuilder::new().map_err(|e| NFAReaderError::NFA(e,0)));
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
        nfa = try!(NFAReader::read_start(nfa, &mut lines));
        nfa = try!(NFAReader::read_finals(nfa, &mut lines));
        for line in lines {
            nfa = try!(NFAReader::read_transition(nfa, line));
        }
        nfa.finalize().map_err(|e| NFAReaderError::NFA(e,0))
    }

    /// Reads a NFA from a `&str`.
    ///
    /// # Description
    ///
    /// * `nfa` - The string representation of the NFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::nfa::reader::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     // (abc)*
    ///     let nfa =
    ///         "0 1\n\
    ///          0 3\n\
    ///          a 0 1\n\
    ///          b 1 2\n\
    ///          c 2 3\n\
    ///          a 3 0";
    ///     let nfa = NFAReader::new_from_string(nfa);
    ///     match nfa {
    ///         Ok(nfa) => {
    ///            // Do stuff with the nfa
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn new_from_string(nfa: &str) -> Result<NFA> {
        NFAReader::new_from_lines(&mut nfa.lines().map(|line| Ok(line.to_string())))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_file() {
        let model =
            "";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::MissingStartingState) => assert!(true),
            _ => assert!(false, "MissingStartingState expected."),
        }
    }

    #[test]
    fn test_start_not_a_number() {
        let model =
            "a";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::Parse(_,line)) => assert!(line == 1),
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
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::Parse(_,line)) => assert!(line == 1),
            _ => assert!(false, "Parse expected."),
        }
    }

    #[test]
    fn test_no_finals() {
        let model =
            "1\n\
            ";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "MissingFinalStates expected."),
        }
    }

    #[test]
    fn test_finals_not_a_number() {
        let model =
            "1\n\
             2 a 3";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::Parse(_,line)) => assert!(line == 2),
            _ => assert!(false, "Parse expected."),
        }
    }

    #[test]
    fn test_no_transistions() {
        let model =
            "0\n\
             3";
        let _nfa = NFAReader::new_from_string(&model).unwrap();
    }

    #[test]
    fn test_transitions_with_at_least_four_elements() {
        let model =
            "0\n\
             3\n\
             a 0 1 8";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::IllformedTransition(line)) => assert!(line == 3),
            _ => assert!(false, "IllformedTransition expected."),
        }
    }

    #[test]
    fn test_transitions_start_with_at_least_two_chars() {
        let model =
            "0\n\
             3\n\
             ab 2 3";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::IllformedTransition(line)) => assert!(line == 3),
            _ => assert!(false, "IllformedTransition expected."),
        }
    }

    #[test]
    fn test_transitions_with_src_not_a_number() {
        let model =
            "0\n\
             3\n\
             c b 3";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::Parse(_,line)) => assert!(line == 3),
            _ => assert!(false, "Parse expected."),
        }
    }

    #[test]
    fn test_transitions_with_dest_not_a_number() {
        let model =
            "0\n\
             3\n\
             c 2 b";
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::Parse(_,line)) => assert!(line == 3),
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
        match NFAReader::new_from_string(model) {
            Err(NFAReaderError::NFA(_,line)) => assert!(line == 4),
            _ => assert!(false, "DuplicatedTransition expected."),
        }
    }

    #[test]
    fn test_read_from_fake_file() {
        let file = "fake.txt";
        match NFAReader::new_from_file(file) {
            Err(NFAReaderError::Io(_)) => assert!(true),
            _ => assert!(false, "Io::Error expected."),
        }
    }
}
