extern crate automata;

//use std::process;
use automata::automata::dfa::*;

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
