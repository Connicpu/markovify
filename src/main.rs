#![feature(plugin)]
#![plugin(regex_macros)]

extern crate rand;
extern crate regex;
extern crate rustc_serialize;
extern crate bincode;

use std::fs::File;
use std::io::{self, Write};
use training::{Trainer, StringTrainer};

pub mod chain;
pub mod training;

fn filename() -> &'static str {
    "chains.markov"
}

fn load_chain() -> chain::Chain {
    let mut chain = chain::Chain::new();
    if let Ok(mut file) = File::open(filename()) {
        chain.load(&mut file)
    }
    chain
}

fn save_chain(chain: &mut chain::Chain) {
    chain.save(&mut File::create(filename()).unwrap());
}

fn generate(chain: &chain::Chain, num_sentences: usize, max_words: usize) {
    for _ in 0..num_sentences {
        print!("{} ", chain.generate_sequence(max_words));
    }
    println!("");
}

fn handle_input(input: &String, chain: &mut chain::Chain) {
    let split: Vec<_> = input.splitn(2, ' ').collect();
    match split.get(0).cloned() {
        Some("generate") => {
            let mut sentences = 2;
            if let Some(args) = split.get(1) {
                if let Ok(num) = args.parse() {
                    sentences = num;
                }
            }
            generate(chain, sentences, 50);
        },
        Some("train") => {
            if let Some(data) = split.get(1) {
                let trainer = StringTrainer::new(data);
                trainer.train(chain);
                chain.clear_empty();
                save_chain(chain);
                println!("saved!");
            } else {
                println!("Why not give me a sentence to train from?");
            }
        },
        cmd => {
            if let Some(cmd) = cmd {
                println!("I don't know what `{}` means", cmd);
            } else {
                println!("Please type a command");
            }
        }
    }
}

fn main() {
    let mut chain = load_chain();

    chain.clear_empty();

    print!("> ");
    io::stdout().flush().unwrap();

    let newline = regex!("[\r\n]+");

    let stdin = io::stdin();
    let mut input = String::new();
    while let Ok(_) = stdin.read_line(&mut input) {
        input = newline.replace(&input, "");

        handle_input(&input, &mut chain);

        input.clear();
        print!("> ");
        io::stdout().flush().unwrap();
    }
}
