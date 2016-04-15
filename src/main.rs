extern crate automata;

//use std::process;
use automata::automata::dfa::*;

fn main() {
    // (ab)*c
    let model =
        "0\n\
         3\n\
         a 0 1\n\
         c 0 3\n\
         b 1 2\n\
         a 2 1\n\
         c 2 3";
    match DFA::new_from_file(&model) {
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
