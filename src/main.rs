#![feature(plugin, slice_patterns, convert)]
#![plugin(regex_macros)]

extern crate rand;
extern crate regex;
extern crate rustc_serialize;
extern crate bincode;

use std::fs::File;
use std::io::{self, Write, BufReader, BufRead};
use training::{Trainer, StrTrainer, MultilineTrainer};

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
        print!("{}", chain.generate_sequence(max_words));
    }
    println!("");
}

fn handle_input(input: &String, chain: &mut chain::Chain) -> bool {
    let split: Vec<_> = input.splitn(2, ' ').collect();
    match split.get(0).cloned() {
        Some("quit") => {
            return false;
        }
        Some("generate") => {
            let mut words = 50;
            if let Some(args) = split.get(1) {
                if let Ok(num) = args.parse() {
                    words = num;
                }
            }
            generate(chain, 1, words);
        }
        Some("save") => {
            save_chain(chain);
            println!("saved!");
        }
        Some("train") => {
            if let Some(data) = split.get(1) {
                let mut trainer = StrTrainer::new(&data);
                trainer.train(chain);
            } else {
                println!("Why not give me a sentence to train from?");
            }
        }
        Some("train-file") => {
            if let Some(filename) = split.get(1) {
                if let Ok(file) = File::open(filename) {
                    let mut line = String::new();
                    let mut trainer = MultilineTrainer::new(chain);

                    let mut reader = BufReader::new(file);
                    while let Ok(n) = reader.read_line(&mut line) {
                        if n == 0 { break; }

                        trainer.next(&line).train(chain);
                        line.clear();
                    }
                } else {
                    println!("I couldn't find {}", filename);
                }
            } else {
                println!("You should give me a filename!");
            }
        }
        Some("train-lines") => {
            let stdin = io::stdin();
            let mut line = String::new();
            let mut trainer = MultilineTrainer::new(chain);
            while let Ok(n) = stdin.read_line(&mut line) {
                if n == 0 || line.find("<<<") == Some(0) {
                    break;
                }

                trainer.next(&line).train(chain);

                line.clear();
            }
        }
        Some("list-choices") => {
            if let Some(args) = split.get(1) {
                let args: Vec<_> = args.split_whitespace().collect();
                if let [arg0, arg1, arg2] = args.as_slice() {
                    let lookup = (arg0, arg1, arg2);
                    if let Some(choices) = chain.lookup_choices(lookup) {
                        for choice in choices.choices.iter() {
                            let name = chain.get_name(choice.item).unwrap();
                            println!("{:?} {} | weight: {}", lookup, name, choice.weight);
                        }
                    } else {
                        println!("I've never seen that combination! You should tell me more :3");
                    }
                } else {
                    println!("Please specify 3 words");
                }
            } else {
                println!("Usage: list-choices word0 word1 word2");
                println!("Use \"\" to represent the beginning");
            }
        }
        cmd => {
            if let Some(cmd) = cmd {
                println!("I don't know what `{}` means", cmd);
            } else {
                println!("Please type a command");
            }
        }
    };
    true
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

        if !handle_input(&input, &mut chain) {
            return;
        }

        input.clear();
        print!("> ");
        io::stdout().flush().unwrap();
    }
}
