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

/// The `ENFAError` type.
#[derive(Debug)]
pub enum ENFAError {
    /// The transition from state `usize` with symbol `char` is defined twice.
    DuplicatedTransition(char,usize),
    /// No final state is specified.
    MissingFinalStates,
    /// No starting state is specified.
    MissingStartingState,
}


impl fmt::Display for ENFAError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ENFAError::DuplicatedTransition(symb,state) => write!(f, "Duplicated transition ('{}',{}).", symb, state),
            ENFAError::MissingFinalStates => write!(f, "Missing final states."),
            ENFAError::MissingStartingState => write!(f, "Missing starting state."),
        }
    }
}

impl error::Error for ENFAError {
    fn description(&self) -> &str {
        match *self {
            ENFAError::DuplicatedTransition(_,_) => "Duplicated transition.", 
            ENFAError::MissingFinalStates => "Missing final states.",
            ENFAError::MissingStartingState => "Missing starting state.",
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// The type `ENFA` represents a NonDeterministic Finite Automaton. The transitions
/// of the automatan are stored in a hashtable.
#[derive(Debug)]
pub struct ENFA {
    transitions   : HashMap<(char,usize),HashSet<usize>>,
    e_transitions : HashMap<usize,HashSet<usize>>,
    start         : usize,
    finals        : HashSet<usize>,
}

/// The `ENFABuilder` follows the builder pattern and allows to create a Deterministic
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
/// use automata::e_nfa::core::*;
///
/// fn main() {
///     // (abc)*
///     let nfa = ENFABuilder::new()
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
/// use automata::e_nfa::core::*;
///
/// fn main() {
///     let nfa = ENFABuilder::new()
///         .add_start(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match nfa {
///         Err(ENFAError::MissingFinalStates) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
/// ```
/// extern crate automata;
///
/// use automata::e_nfa::core::*;
///
/// fn main() {
///     let nfa = ENFABuilder::new()
///         .add_final(4)
///         .add_transition('t', 0, 1)
///         .finalize();
///     match nfa {
///         Err(ENFAError::MissingStartingState) => assert!(true),
///         _ => assert!(false),
///     }
/// }
/// ```
///
#[derive(Debug)]
pub struct ENFABuilder {
    transitions   : HashMap<(char,usize),HashSet<usize>>,
    e_transitions : HashMap<usize,HashSet<usize>>,
    start         : Option<usize>,
    finals        : HashSet<usize>,
}

/// Alias for result::Result<T,ENFAError>.
pub type Result<T> = result::Result<T,ENFAError>;

/// ENFABuilding is the trait assiociated to the ENFABuilder type. Each ENFABuilder
/// should implement ENFABuilding trait.
///
/// ENFABuilder can generate some errors during the building stage. For instance,
/// one could try to insert a transition with two different destination states.
///
/// #Errors
///
/// If self contains a ENFAerror then each function should transfer this error.
pub trait ENFABuilding {
    /// Add a starting state to the ENFA.
    ///
    /// # Errors
    /// 
    /// In the futur will return a ENFAError::DuplicatedStartingState if
    /// two starting states are added.
    fn add_start(self, state: usize) -> Result<ENFABuilder>;

    /// Add a final state to the ENFA.
    fn add_final(self, state: usize) -> Result<ENFABuilder>;

    /// Add a transition to the ENFA.
    ///
    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<ENFABuilder>;

    /// Add an epsilon transition to the ENFA.
    ///
    fn add_e_transition(self, src: usize, dest: usize) -> Result<ENFABuilder>;

    /// Finalize the building of the ENFA.
    ///
    /// # Errors
    ///
    /// Return a ENFAError::MissingStartingState if no starting state is specified.
    ///
    /// Return a ENFAError::MissingFinalStates if no final state is specified.
    fn finalize(self) -> Result<ENFA>;
}

impl ENFABuilder {
    /// Creates a new ENFABuilder.
    pub fn new() -> Result<ENFABuilder> {
        Ok(ENFABuilder{
            transitions: HashMap::new(),
            e_transitions: HashMap::new(),
            start: None,
            finals: HashSet::new()
        })
    }
}

impl ENFABuilding for ENFABuilder {
    fn add_start(self, state: usize) -> Result<ENFABuilder> {
        Ok(self).add_start(state)
    }

    fn add_final(self, state: usize) -> Result<ENFABuilder> {
        Ok(self).add_final(state)
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<ENFABuilder> {
        Ok(self).add_transition(symb,src,dest)
    }

    fn add_e_transition(self, src: usize, dest: usize) -> Result<ENFABuilder> {
        Ok(self).add_e_transition(src,dest)
    }

    fn finalize(self) -> Result<ENFA> {
        Ok(self).finalize()
    }
}


/// Implementing ENFABuilding trait for Result<ENFABuilder> allows
/// to chain the return value of the ENFABuilder instead of unwrapping them
/// at each stage of the building process.
impl ENFABuilding for Result<ENFABuilder> {
    fn add_start(self, state: usize) -> Result<ENFABuilder> {
        self.and_then(|mut nfa| {
            nfa.start = Some(state);
            Ok(nfa)
        })
    }

    fn add_final(self, state: usize) -> Result<ENFABuilder> {
        self.and_then(|mut nfa| {
            nfa.finals.insert(state);
            Ok(nfa)
        })
    }

    fn add_transition(self, symb: char, src: usize, dest: usize) -> Result<ENFABuilder> {
        self.and_then(|mut nfa| {
            {
                // A block is mandatory here because states borrow a value inside nfa.
                // Ok(nfa) moves nfa but if states is in the same block it will has the
                // same lifetime and it's not possible to move a borrowed value.
                let states = nfa.transitions.entry((symb,src)).or_insert(HashSet::new());
                (*states).insert(dest);
            }
            Ok(nfa)
        })
    }

    fn add_e_transition(self, src: usize, dest: usize) -> Result<ENFABuilder> {
        self.map(|mut nfa| {
            {
                // A block is mandatory here because states borrow a value inside nfa.
                // Ok(nfa) moves nfa but if states is in the same block it will has the
                // same lifetime and it's not possible to move a borrowed value.
                let states = nfa.e_transitions.entry(src).or_insert(HashSet::new());
                (*states).insert(dest);
            }
            nfa
        })
    }

    fn finalize(self) -> Result<ENFA> {
        self.and_then(|nfa| {
            if nfa.start.is_none() {
                Err(ENFAError::MissingStartingState)
            } else if nfa.finals.is_empty() {
                Err(ENFAError::MissingFinalStates)
            } else {
                Ok(ENFA{
                    transitions: nfa.transitions,
                    e_transitions: nfa.e_transitions,
                    start: nfa.start.unwrap(),
                    finals: nfa.finals
                })
            }
        })
    }
}

impl ENFA {
    /// Test if an input string is a word of the language defined by the ENFA.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate automata;
    ///
    /// use automata::e_nfa::core::*;
    /// 
    /// fn main() {
    ///     // (abc)*
    ///     let nfa = ENFABuilder::new()
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
                                //.map(|nexts| {
                                    //self.e_transitions
                                        //.get(&*state)
                                        //.map(|trans| nexts.union(nexts).cloned.collect())
                                //})
                        })
                    })
                })
            })
            .unwrap_or(HashSet::new())
            .intersection(&self.finals)
            .next().is_some()
    }
}

impl fmt::Display for ENFA {
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
        for (tr,d) in self.e_transitions.iter() {
            try!(writeln!(f, "  {} => {:?}", tr, d));
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa() {
        let nfa = ENFABuilder::new()
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
        let _nfa = ENFABuilder::new()
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
        let nfa = ENFABuilder::new()
            .add_final(3)
            .add_transition('a', 0, 1)
            .finalize();
        match nfa {
            Err(ENFAError::MissingStartingState) => assert!(true),
            _ => assert!(false, "MissingStartingState expected."),
        }
    }

    #[test]
    fn test_nfa_builder_missing_finals() {
        let nfa = ENFABuilder::new()
            .add_start(0)
            .add_transition('a', 0, 1)
            .finalize();
        match nfa {
            Err(ENFAError::MissingFinalStates) => assert!(true),
            _ => assert!(false, "MissingFinalStates expected."),
        }
    }
}
