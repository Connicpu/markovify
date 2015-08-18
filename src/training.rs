use chain::Chain;
use chain::State;

pub trait Trainer {
    fn train(&mut self, chain: &mut Chain);
}

pub struct StrTrainer<'a> {
    data: &'a str,
}

impl<'a> StrTrainer<'a> {
    pub fn new(data: &'a str) -> Self {
        StrTrainer {
            data: data
        }
    }
}

impl<'a> Trainer for StrTrainer<'a> {
    fn train(&mut self, chain: &mut Chain) {
        let mut state = chain.begin();
        for word in self.data.split_whitespace() {
            let next = chain.push_word(word);
            chain.train_choice(state, next);
            state.push(next);
        }
    }
}

pub struct MultilineTrainer {
    state: State,
    begin: State,
}

pub struct MultilineEntry<'a, 'b> {
    data: &'a str,
    state: &'b mut State,
    ext_state: State,
}

impl MultilineTrainer {
    pub fn new(chain: &Chain) -> Self {
        MultilineTrainer {
            state: chain.begin(),
            begin: chain.begin(),
        }
    }

    pub fn next<'a, 'b>(&'b mut self, data: &'a str) -> MultilineEntry<'a, 'b> {
        MultilineEntry {
            data: data,
            state: &mut self.state,
            ext_state: self.begin,
        }
    }
}

impl<'a, 'b> Trainer for MultilineEntry<'a, 'b> {
    fn train(&mut self, chain: &mut Chain) {
        let begin = self.ext_state;
        for word in self.data.split_whitespace() {
            let next = chain.push_word(word);

            chain.train_choice(*self.state, next);
            if *self.state != self.ext_state {
                chain.train_choice(self.ext_state, next);
            }

            self.state.push(next);
            self.ext_state.push(next);
        }
        self.ext_state = begin;
    }
}
