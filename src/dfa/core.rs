// Copyright 2016 Vincent Vigneron. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at.your option.
// This file may not be copied, modified, or distributed
// except according to those terms.

extern crate itertools;

use std::collections::{HashSet,HashMap};
use std::fmt;                          // Formatter, format!, Display, Debug, write!
use std::error;
use std::result;

/// The `DFAError` type.
#[derive(Debug)]
pub enum DFAError {
    /// The transition from state `usize` with symbol `char` is defined twice.
    DuplicatedTransition(char,usize),
    /// No final state is specified.
    MissingFinalStates,
    /// No starting state is specified.
    MissingStartingState,
}


impl fmt::Display for DFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DFAError::DuplicatedTransition(symb,state) => write!(f, "Duplicated transition ('{}',{}).", symb, state),
            DFAError::MissingFinalStates => write!(f, "Missing final states."),
            DFAError::MissingStartingState => write!(f, "Missing starting state."),
        }
    }
}

impl error::Error for DFAError {
    fn description(&self) -> &str {
        match *self {
            DFAError::DuplicatedTransition(_,_) => "Duplicated transition.", 
            DFAError::MissingFinalStates => "Missing final states.",
            DFAError::MissingStartingState => "Missing starting state.",
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// The type `DFA` represents a Deterministic Finite Automaton. The transitions
/// of the automatan are stored in a hashtable.
#[derive(Debug)]
pub struct DFA {
    transitions : HashMap<(char,usize),usize>,
    start       : usize,
    finals      : HashSet<usize>,
}

/// The `DFABuilder` follows the builder pattern and allows to create a Deterministic
/// Finite Automaton. The builder is moved at each call so it is necessary to bind
/// to a new variable the return value for each function of the builder.
///
/// # Errors
///
/// Return an error if the starting state is not specified.
///
/// Return an error if the final states are not specified.
///
/// # Examples
///
/// ```
/// extern crate automata;
///
/// use automata::dfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     // (abc)*
///     let dfa = DFABuilder::new()
///         .add_start(0)
///         .add_final(0)
///         .add_transition('a', 0, 1)
///         .add_transition('b', 1, 2)
///         .add_transition('c', 2, 0)
///         .finalize();
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::dfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     let dfa = DFABuilder::new()
///         .add_start(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match dfa {
///         Err(DFAError::MissingFinalStates) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::dfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     let dfa = DFABuilder::new()
///         .add_start(4)
///         .add_transition('t', 0, 1)
///         .add_transition('t', 0, 2)
///         .finalize();
///     match dfa {
///         Err(DFAError::DuplicatedTransition(symb,src)) => assert!((symb,src)==('t',0)),
///         _ => assert!(false),
///     }
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::dfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     let dfa = DFABuilder::new()
///         .add_final(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match dfa {
///         Err(DFAError::MissingStartingState) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
#[derive(Debug)]
pub struct DFABuilder {
    transitions : HashMap<(char,usize),usize>,
    start       : Option<usize>,
    finals      : HashSet<usize>,
}

/// Alias for result::Result<T,DFAError>.
pub type Result<T> = result::Result<T,DFAError>;

/// DFABuilding is the trait assiociated to the DFABuilder type. Each DFABuilder
/// should implement DFABuilding trait.
///
/// DFABuilder can generate some errors during the building stage. For instance,
/// one could try to insert a transition with two different destination states.
///
/// #Errors
///
/// If self contains a DFAerror then each function should transfer this error.
pub trait DFABuilding {
    /// Add a starting state to the DFA.
    ///
    /// # Errors
    /// 
    /// In the futur will return a DFAError::DuplicatedStartingState if
    /// two starting states are added.
    fn add_start(mut self, state: usize) -> Result<DFABuilder>;

    /// Add a final state to the DFA.
    fn add_final(mut self, state: usize) -> Result<DFABuilder>;

    /// Add a transition to the DFA.
    ///
    /// # Errors
    ///
    /// Return a DFAError::DuplicatedTransition(symb,src) if a transtion
    /// with the same symb and src has already been inserted, even if
    /// the destination state is the same.
    fn add_transition(mut self, symb: char, src: usize, dest: usize) -> Result<DFABuilder>;

    /// Finalize the building of the DFA.
    ///
    /// # Errors
    ///
    /// Return a DFAError::MissingStartingState if no starting state is specified.
    ///
    /// Return a DFAError::MissingFinalStates if no final state is specified.
    fn finalize(self) -> Result<DFA>;
}

impl DFABuilder {
    /// Creates a new DFABuilder.
    pub fn new() -> Result<DFABuilder> {
        Ok(DFABuilder{transitions: HashMap::new(), start: None, finals: HashSet::new()})
    }
}

impl DFABuilding for DFABuilder {
    fn add_start(self, state: usize) -> Result<DFABuilder> {
        Ok(self).add_start(state)
    }

    fn add_final(self, state: usize) -> Result<DFABuilder> {
        Ok(self).add_final(state)
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<DFABuilder> {
        Ok(self).add_transition(symb,src,dest)
    }

    fn finalize(self) -> Result<DFA> {
        Ok(self).finalize()
    }
}


/// Implementing DFABuilding trait for Result<DFABuilder> allows
/// to chain the return value of the DFABuilder instead of unwrapping them
/// at each stage of the building process.
impl DFABuilding for Result<DFABuilder> {
    fn add_start(self, state: usize) -> Result<DFABuilder> {
        self.and_then(|mut dfa| {
            dfa.start = Some(state);
            Ok(dfa)
        })
    }

    fn add_final(self, state: usize) -> Result<DFABuilder> {
        self.and_then(|mut dfa| {
            dfa.finals.insert(state);
            Ok(dfa)
        })
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<DFABuilder> {
        self.and_then(|mut dfa| {
            if dfa.transitions.insert((symb,src), dest).is_some() {
                return Err(DFAError::DuplicatedTransition(symb,src));
            }
            Ok(dfa)
        })
    }

    fn finalize(self) -> Result<DFA> {
        self.and_then(|dfa| {
            if dfa.start.is_none() {
                Err(DFAError::MissingStartingState)
            } else if dfa.finals.is_empty() {
                Err(DFAError::MissingFinalStates)
            } else {
                Ok(DFA{transitions: dfa.transitions, start: dfa.start.unwrap(), finals: dfa.finals})
            }
        })
    }
}

impl DFA {
    /// Test if an input string is a word of the language defined by the DFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::dfa::core::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     // (abc)*
    ///     let dfa = DFABuilder::new()
    ///         .add_start(0)
    ///         .add_final(0)
    ///         .add_transition('a', 0, 1)
    ///         .add_transition('b', 1, 2)
    ///         .add_transition('c', 2, 0)
    ///         .finalize();
    ///     match dfa {
    ///         Ok(dfa) => {
    ///            assert!(dfa.test("abc"));
    ///            assert!(dfa.test(""));
    ///            assert!(!dfa.test("a"));
    ///            assert!(!dfa.test("ab"));
    ///            assert!(!dfa.test("abca"));
    ///            assert!(!dfa.test("abcab"));
    ///            assert!(dfa.test("abcabcabc"));
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn test(&self, input: &str) -> bool {
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
        let dfa = DFABuilder::new()
            .add_start(0)
            .add_final(3)
            .add_transition('a', 0, 1)
            .add_transition('c', 0, 3)
            .add_transition('b', 1, 2)
            .add_transition('a', 2, 1)
            .add_transition('c', 2, 3)
            .finalize()
            .unwrap();
        let samples =
            vec![("ababac", false),
                 ("ababc", true),
                 ("", false),
                 ("abc", true),
                 ("c", true),
                 ("ac", false),
                 ("ababababababababababababababababababababc", true),];

        for (input,expected_result) in samples {
            assert!(dfa.test(input) == expected_result, "input false for: \"{}\"", input);
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
            .finalize()
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
            .add_transition('a', 0, 2)
            .finalize();
        match dfa {
            Err(DFAError::DuplicatedTransition(sy,sr)) => assert!((sy,sr) == ('a',0)),
            _ => assert!(false, "DuplicatedTransition expected."),
        }
    }

    #[test]
    fn test_dfa_builder_missing_start() {
        let dfa = DFABuilder::new()
            .add_final(3)
            .add_transition('a', 0, 1)
            .finalize();
        match dfa {
            Err(DFAError::MissingStartingState) => assert!(true),
            _ => assert!(false, "MissingStartingState expected."),
        }
    }

    #[test]
    fn test_dfa_builder_missing_finals() {
        let dfa = DFABuilder::new()
            .add_start(0)
            .add_transition('a', 0, 1)
            .finalize();
        match dfa {
            Err(DFAError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "MissingFinalStates expected."),
        }
    }
}
