// Copyright 2016 Vincent Vigneron. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at.your option.
// This file may not be copied, modified, or distributed
// except according to those terms.

extern crate automaton;

use std::process;
use automaton::dfa::core::*;

fn main() {
    // (toto)*
    let dfa = DFABuilder::new()
        .add_start(0)
        .add_final(0)
        .add_transition('t', 0, 1)
        .add_transition('o', 1, 2)
        .add_transition('t', 2, 3)
        .add_transition('o', 3, 0)
        .finalize();

    match dfa {
        Ok(dfa) => {
            println!("{}", dfa);
            println!("{:?}", dfa.test("toto"));
            println!("{:?}", dfa.test(""));
            println!("{:?}", dfa.test("t"));
            println!("{:?}", dfa.test("to"));
            println!("{:?}", dfa.test("tot"));
            println!("{:?}", dfa.test("totot"));
            println!("{:?}", dfa.test("totototo"));
        }
        _ => process::exit(1),
    }
}
