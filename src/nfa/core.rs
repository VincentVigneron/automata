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

/// The `NFAError` type.
#[derive(Debug)]
pub enum NFAError {
    /// The transition from state `usize` with symbol `char` is defined twice.
    DuplicatedTransition(char,usize),
    /// No final state is specified.
    MissingFinalStates,
    /// No starting state is specified.
    MissingStartingState,
}


impl fmt::Display for NFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NFAError::DuplicatedTransition(symb,state) => write!(f, "Duplicated transition ('{}',{}).", symb, state),
            NFAError::MissingFinalStates => write!(f, "Missing final states."),
            NFAError::MissingStartingState => write!(f, "Missing starting state."),
        }
    }
}

impl error::Error for NFAError {
    fn description(&self) -> &str {
        match *self {
            NFAError::DuplicatedTransition(_,_) => "Duplicated transition.", 
            NFAError::MissingFinalStates => "Missing final states.",
            NFAError::MissingStartingState => "Missing starting state.",
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// The type `NFA` represents a NonDeterministic Finite Automaton. The transitions
/// of the automatan are stored in a hashtable.
#[derive(Debug)]
pub struct NFA {
    transitions : HashMap<(char,usize),HashSet<usize>>,
    start       : usize,
    finals      : HashSet<usize>,
}

/// The `NFABuilder` follows the builder pattern and allows to create a Deterministic
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
/// use automata::nfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     // (abc)*
///     let nfa = NFABuilder::new()
///         .add_start(0)
///         .add_final(3)
///         .add_final(0)
///         .add_transition('a', 0, 1)
///         .add_transition('b', 1, 2)
///         .add_transition('c', 2, 3)
///         .add_transition('a', 3, 1)
///         .finalize();
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::nfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     let nfa = NFABuilder::new()
///         .add_start(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match nfa {
///         Err(NFAError::MissingFinalStates) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::nfa::core::*;
/// use std::error::Error;
/// 
/// fn main() {
///     let nfa = NFABuilder::new()
///         .add_final(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match nfa {
///         Err(NFAError::MissingStartingState) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
#[derive(Debug)]
pub struct NFABuilder {
    transitions : HashMap<(char,usize),HashSet<usize>>,
    start       : Option<usize>,
    finals      : HashSet<usize>,
}

/// Alias for result::Result<T,NFAError>.
pub type Result<T> = result::Result<T,NFAError>;

/// NFABuilding is the trait assiociated to the NFABuilder type. Each NFABuilder
/// should implement NFABuilding trait.
///
/// NFABuilder can generate some errors during the building stage. For instance,
/// one could try to insert a transition with two different destination states.
///
/// #Errors
///
/// If self contains a NFAerror then each function should transfer this error.
pub trait NFABuilding {
    /// Add a starting state to the NFA.
    ///
    /// # Errors
    /// 
    /// In the futur will return a NFAError::DuplicatedStartingState if
    /// two starting states are added.
    fn add_start(self, state: usize) -> Result<NFABuilder>;

    /// Add a final state to the NFA.
    fn add_final(self, state: usize) -> Result<NFABuilder>;

    /// Add a transition to the NFA.
    ///
    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<NFABuilder>;

    /// Finalize the building of the NFA.
    ///
    /// # Errors
    ///
    /// Return a NFAError::MissingStartingState if no starting state is specified.
    ///
    /// Return a NFAError::MissingFinalStates if no final state is specified.
    fn finalize(self) -> Result<NFA>;
}

impl NFABuilder {
    /// Creates a new NFABuilder.
    pub fn new() -> Result<NFABuilder> {
        Ok(NFABuilder{transitions: HashMap::new(), start: None, finals: HashSet::new()})
    }
}

impl NFABuilding for NFABuilder {
    fn add_start(self, state: usize) -> Result<NFABuilder> {
        Ok(self).add_start(state)
    }

    fn add_final(self, state: usize) -> Result<NFABuilder> {
        Ok(self).add_final(state)
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<NFABuilder> {
        Ok(self).add_transition(symb,src,dest)
    }

    fn finalize(self) -> Result<NFA> {
        Ok(self).finalize()
    }
}


/// Implementing NFABuilding trait for Result<NFABuilder> allows
/// to chain the return value of the NFABuilder instead of unwrapping them
/// at each stage of the building process.
impl NFABuilding for Result<NFABuilder> {
    fn add_start(self, state: usize) -> Result<NFABuilder> {
        self.map(|mut nfa| {
            nfa.start = Some(state);
            nfa
        })
    }

    fn add_final(self, state: usize) -> Result<NFABuilder> {
        self.map(|mut nfa| {
            nfa.finals.insert(state);
            nfa
        })
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<NFABuilder> {
        self.map(|mut nfa| {
            {
                // `states` is a mutable reference to a value inside `transitions` (see or_insert).
                // It 's not possible to return nfa if states is in the same scope, because the
                // return statement will try to move nfa, including the `transitions` field while
                // states will still have a mutable refrence of `transitions`.
                let states = nfa.transitions.entry((symb,src)).or_insert(HashSet::new());
                (*states).insert(dest);
            }
            nfa
        })
    }

    fn finalize(self) -> Result<NFA> {
        self.and_then(|nfa| {
            if nfa.start.is_none() {
                Err(NFAError::MissingStartingState)
            } else if nfa.finals.is_empty() {
                Err(NFAError::MissingFinalStates)
            } else {
                Ok(NFA{transitions: nfa.transitions, start: nfa.start.unwrap(), finals: nfa.finals})
            }
        })
    }
}

impl NFA {
    /// Test if an input string is a word of the language defined by the NFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::nfa::core::*;
    /// use std::error::Error;
    /// 
    /// fn main() {
    ///     // (abc)*
    ///     let nfa = NFABuilder::new()
    ///         .add_start(0)
    ///         .add_final(3)
    ///         .add_final(0)
    ///         .add_transition('a', 0, 1)
    ///         .add_transition('b', 1, 2)
    ///         .add_transition('c', 2, 3)
    ///         .add_transition('a', 3, 1)
    ///         .finalize();
    ///     match nfa {
    ///         Ok(nfa) => {
    ///            assert!(nfa.test("abc"));
    ///            assert!(nfa.test(""));
    ///            assert!(!nfa.test("a"));
    ///            assert!(!nfa.test("ab"));
    ///            assert!(!nfa.test("abca"));
    ///            assert!(!nfa.test("abcab"));
    ///            assert!(nfa.test("abcabcabc"));
    ///         },
    ///         Err(e) => println!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn test(&self, input: &str) -> bool {
        let start : HashSet<_> = [self.start].iter().cloned().collect();
        input
            .chars()
            .fold(Some(start), |states,c| {
                states.and_then(|states| {
                    states.iter().fold(Some(HashSet::new()), |acc, state| {
                        acc.and_then(|acc| {
                            self.transitions
                                .get(&(c,*state))
                                .map(|trans| acc.union(trans).cloned().collect())
                        })
                    })
                })
            })
            .unwrap_or(HashSet::new())
            .intersection(&self.finals)
            .next().is_some()
    }
}

impl fmt::Display for NFA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "START: {}", self.start));
        try!(writeln!(f, "FINALS:"));
        for fi in self.finals.iter() {
            try!(writeln!(f,"  {}", fi));
        }
        try!(writeln!(f, "TRANSITIONS:"));
        for (tr,d) in self.transitions.iter() {
            let (c,s) = *tr;
            try!(writeln!(f, "  ({},{}) => {:?}", c, s, d));
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa() {
        let nfa = NFABuilder::new()
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
            assert!(nfa.test(input) == expected_result, "input false for: \"{}\"", input);
        }
    }

    #[test]
    fn test_nfa_builder() {
        let _nfa = NFABuilder::new()
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
    fn test_nfa_builder_missing_start() {
        let nfa = NFABuilder::new()
            .add_final(3)
            .add_transition('a', 0, 1)
            .finalize();
        match nfa {
            Err(NFAError::MissingStartingState) => assert!(true),
            _ => assert!(false, "MissingStartingState expected."),
        }
    }

    #[test]
    fn test_nfa_builder_missing_finals() {
        let nfa = NFABuilder::new()
            .add_start(0)
            .add_transition('a', 0, 1)
            .finalize();
        match nfa {
            Err(NFAError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "MissingFinalStates expected."),
        }
    }
}
