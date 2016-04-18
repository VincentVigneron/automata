extern crate automata;

use std::process;
use automata::automata::dfa::*;

fn main() {
    // (toto)*
    let dfa = DFABuilder::new()
        .add_start(0)
        .add_final(4)
        .add_final(0)
        .add_transition('t', 0, 1)
        .add_transition('o', 1, 2)
        .add_transition('t', 2, 3)
        .add_transition('o', 3, 4)
        .add_transition('t', 4, 1)
        .finalize();
    match dfa {
        Ok(dfa) => {
            println!("{}", dfa);
            println!("{:?}", dfa.run("toto"));
            println!("{:?}", dfa.run(""));
            println!("{:?}", dfa.run("t"));
            println!("{:?}", dfa.run("to"));
            println!("{:?}", dfa.run("tot"));
            println!("{:?}", dfa.run("totot"));
            println!("{:?}", dfa.run("totototo"));
        },
        _ => process::exit(1),
    }
}

/*
fn main() {
    // (ab)*c
    let file = "data/dfa1.txt";
    match DFA::new_from_file(file) {
        Ok(d) => {
            let dfa = d;
            println!("{}", dfa);
            println!("{:?}", dfa.run("ababac"));
            println!("{:?}", dfa.run("ababc"));
            println!("{:?}", dfa.run(""));
            println!("{:?}", dfa.run("abc"));
            println!("{:?}", dfa.run("ac"));
            println!("{:?}", dfa.run("c"));
            println!("{:?}", dfa.run("ababababababababababababababababababababc"));
        },
        Err(e) => {
            println!("{}", e);
            //process::exit(0)
        },
    }
}
*/
