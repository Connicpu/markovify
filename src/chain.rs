use std;
use std::collections::HashMap;
use std::default::Default;
use std::io::{Read, Write};
use std::mem::replace;
use rand::distributions::Sample;
use rand::distributions::range::Range;
use rand::{Rng, thread_rng};
use bincode::rustc_serialize::{encode_into, decode_from};
use bincode::SizeLimit::Infinite;

#[derive(Copy, Clone, Debug, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct WordId(u32);

#[derive(Copy, Clone, Debug, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(RustcEncodable, RustcDecodable)]
pub struct State(WordId, WordId, WordId);

pub type ChoiceLookup<'a> = (&'a str, &'a str, &'a str);

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Chain {
    words: Vec<String>,
    word_lookup: HashMap<String, WordId>,
    graph: MarkovGraph,
}

#[derive(Default, Clone, RustcEncodable, RustcDecodable)]
struct MarkovGraph {
    edges: HashMap<State, Choices>,
}

#[derive(Default, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Choices {
    pub choices: Vec<ChoiceWeight>,
    pub total: u32,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct ChoiceWeight {
    pub item: WordId,
    pub weight: u32,
}

pub struct GeneratingIterator<'a> {
    state: State,
    rng: ::rand::ThreadRng,
    chain: &'a Chain,
}

impl State {
    pub fn push(&mut self, next: WordId) {
        let next = State(self.1, self.2, next);
        replace(self, next);
    }
}

impl Chain {
    pub fn new() -> Chain {
        Chain {
            words: vec!["".into()],
            word_lookup: {
                let mut map = HashMap::new();
                map.insert("".into(), WordId(0));
                map.insert("\"\"".into(), WordId(0));
                map
            },
            graph: Default::default(),
        }
    }

    pub fn begin(&self) -> State {
        let begin = WordId(0);
        State(begin, begin, begin)
    }

    pub fn push_word(&mut self, word: &str) -> WordId {
        if let Some(&id) = self.word_lookup.get(word) {
            return id;
        }

        if self.words.len() >= std::u32::MAX as usize {
            panic!("TOO MANY WORDS! AHHHHH! HOW DID YOU MANAGE THAT?!");
        }

        let id = WordId(self.words.len() as u32);
        self.words.push(word.into());
        self.word_lookup.insert(word.into(), id);
        id
    }

    pub fn find_word(&self, word: &str) -> Option<WordId> {
        self.word_lookup.get(word).cloned()
    }

    pub fn get_name(&self, word: WordId) -> Option<&str> {
        self.words.get(word.0 as usize).map(|s| s as &str)
    }

    pub fn lookup_choices<'a, 'b>(&'a self, prefix: ChoiceLookup<'b>) -> Option<&'a Choices> {
        match (self.find_word(prefix.0),
               self.find_word(prefix.1),
               self.find_word(prefix.2)) {
            (Some(w0), Some(w1), Some(w2)) => self.graph.edges.get(&State(w0, w1, w2)),
            result => {
                if result.0 == None {
                    println!("I don't know {}", prefix.0);
                }
                if result.1 == None {
                    println!("I don't know {}", prefix.1);
                }
                if result.2 == None {
                    println!("I don't know {}", prefix.2);
                }
                None
            }
        }
    }

    pub fn train_choice(&mut self, prefix: State, suffix: WordId) {
        let choices = self.graph.edges.entry(prefix).or_insert(Default::default());

        let needs_push = if let Some(choice) = choices.choices
                                                      .iter_mut()
                                                      .find(|weighted| weighted.item == suffix) {
            choice.weight += 1;
            false
        } else {
            true
        };

        if needs_push {
            choices.choices.push(ChoiceWeight {
                item: suffix,
                weight: 1,
            });
        }

        choices.total += 1;
    }

    pub fn iter(&self) -> GeneratingIterator {
        GeneratingIterator {
            state: self.begin(),
            rng: thread_rng(),
            chain: self,
        }
    }

    pub fn next_word<'a, R: Rng>(&'a self, state: &mut State, rng: &mut R) -> Option<&'a str> {
        let choices = match self.graph.edges.get(&state) {
            Some(choices) => choices,
            None => return None,
        };

        if choices.choices.len() == 0 {
            return None;
        }

        let mut range = Range::new(0, choices.total);
        let mut selector = range.sample(rng);

        let mut choice_idx = 0;
        for (i, v) in choices.choices.iter().enumerate() {
            if selector < v.weight {
                choice_idx = i;
                break;
            }

            selector -= v.weight;
        }

        let choice = choices.choices[choice_idx].item;
        state.push(choice);

        Some(&self.words[choice.0 as usize])
    }

    pub fn generate_sequence(&self, max_length: usize) -> String {
        let mut sequence = String::new();

        let mut rng = thread_rng();
        let mut state = self.begin();

        for _ in 0..max_length {
            let word = match self.next_word(&mut state, &mut rng) {
                Some(word) => word,
                None => break,
            };

            sequence = sequence + word;
            sequence.push(' ');
        }

        sequence
    }

    pub fn save<W: Write>(&self, writer: &mut W) {
        encode_into(self, writer, Infinite).unwrap();
    }

    pub fn load<R: Read>(&mut self, reader: &mut R) {
        replace(self, decode_from(reader, Infinite).unwrap());

        if self.find_word("\"\"") == None {
            self.word_lookup.insert("\"\"".into(), WordId(0));
        }
    }

    pub fn clear_empty(&mut self) {
        let begin = self.begin();
        if let Some(begin) = self.graph.edges.get_mut(&begin) {
            let amount = {
                let choice = begin.choices.iter_mut().find(|choice| choice.item == WordId(1));
                if let Some(choice) = choice {
                    let old = choice.weight;
                    choice.weight = 0;
                    old
                } else {
                    0
                }
            };
            begin.total -= amount;
        }
    }

    pub fn print_stats(&self) {
        println!("Vocabulary: {} words", self.words.len());
        println!("Known prefixes: {}", self.graph.edges.len());
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::new()
    }
}

impl<'a> Iterator for GeneratingIterator<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.chain.next_word(&mut self.state, &mut self.rng)
    }
}
