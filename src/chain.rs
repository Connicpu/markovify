use std;
use std::collections::HashMap;
use std::default::Default;
use std::io::{Read, Write};
use std::mem::replace;
use rand::distributions::Sample;
use rand::distributions::range::Range;
use rand::thread_rng;
use bincode::rustc_serialize::{encode_into, decode_from};
use bincode::SizeLimit::Infinite;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, RustcEncodable, RustcDecodable)]
pub struct WordId(u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, RustcEncodable, RustcDecodable)]
pub struct State(WordId, WordId, WordId);

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

#[derive(Default, Clone, RustcEncodable, RustcDecodable)]
struct Choices {
    choices: Vec<ChoiceWeight>,
    total: u32,
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
struct ChoiceWeight {
    item: WordId,
    weight: u32,
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
                map
            },
            graph: Default::default()
        }
    }

    pub fn begin(&self) -> State {
        let begin = WordId(0);
        State(begin, begin, begin)
    }

    pub fn push_word(&mut self, word: &str) -> WordId {
        if let Some(&id) = self.word_lookup.get(word) {
            return id
        }

        if self.words.len() > std::u32::MAX as usize {

        }

        let id = WordId(self.words.len() as u32);
        self.words.push(word.into());
        self.word_lookup.insert(word.into(), id);
        id
    }

    pub fn find_word(&self, word: &str) -> Option<WordId> {
        self.word_lookup.get(word).cloned()
    }

    pub fn train_choice(&mut self, prefix: State, suffix: WordId) {
        let choices = self.graph.edges.entry(prefix).or_insert(Default::default());

        let needs_push = if let Some(choice) = choices.choices.iter_mut().find(|weighted| weighted.item == suffix) {
            choice.weight += 1;
            false
        } else {
            true
        };

        if needs_push {
            choices.choices.push(ChoiceWeight {
                item: suffix,
                weight: 1
            });
        }

        choices.total += 1;
    }

    pub fn generate_sequence(&self, max_length: usize) -> String {
        let mut sequence = String::new();

        let mut rng = thread_rng();
        let mut state = self.begin();

        for _ in 0..max_length {
            let choices = match self.graph.edges.get(&state) {
                Some(choices) => choices,
                None => break,
            };

            if choices.choices.len() == 0 {
                break;
            }

            let mut range = Range::new(0, choices.total);
            let mut selector = range.sample(&mut rng);

            let mut choice_idx = 0;
            for (i, v) in choices.choices.iter().enumerate() {
                if selector < v.weight {
                    choice_idx = i;
                    break;
                }

                selector -= v.weight;
            }

            let choice = choices.choices[choice_idx].item;

            sequence = sequence + &self.words[choice.0 as usize];
            sequence.push(' ');
            state.push(choice);
        }

        sequence
    }

    pub fn save<W: Write>(&self, writer: &mut W) {
        encode_into(self, writer, Infinite).unwrap();
    }

    pub fn load<R: Read>(&mut self, reader: &mut R) {
        replace(self, decode_from(reader, Infinite).unwrap());
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
}

impl Default for Chain {
    fn default() -> Self {
        Chain::new()
    }
}
