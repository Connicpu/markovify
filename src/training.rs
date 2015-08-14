use chain::Chain;
use std::mem::replace;

pub trait Trainer {
    fn train(&self, chain: &mut Chain);
}

pub struct StringTrainer {
    sentences: Vec<Sentence>,
}

struct Sentence {
    words: Vec<String>
}

impl StringTrainer {
    pub fn new(data: &str) -> StringTrainer {
        let mut sentences = Vec::new();
        let mut sentence = Vec::new();

        let ignore = regex!(r"[,]");
        let end_of_sentence = regex!(r"[\.\?!=\+;-]");

        for word in data.split_whitespace() {
            let word = ignore.replace(word, "").to_lowercase();

            if end_of_sentence.is_match(&word) {
                sentence.push(end_of_sentence.replace(&word, ""));
                sentences.push(Sentence { words: replace(&mut sentence, Vec::new()) });
            } else {
                sentence.push(word);
            }
        }

        sentences.push(Sentence { words: sentence });

        StringTrainer { sentences: sentences }
    }
}

impl Trainer for StringTrainer {
    fn train(&self, chain: &mut Chain) {
        let begin = chain.begin();
        let end = chain.end();

        for sentence in self.sentences.iter() {
            let mut state = (begin, begin);

            for word in sentence.words.iter() {
                let next = chain.push_word(word.clone());
                chain.train_choice(state, next);
                state = (state.1, next);
            }

            chain.train_choice(state, end)
        }
    }
}
