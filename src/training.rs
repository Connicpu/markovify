use chain::Chain;

pub trait Trainer {
    fn train(&self, chain: &mut Chain);
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
    fn train(&self, chain: &mut Chain) {
        let mut state = chain.begin();
        for word in self.data.split_whitespace() {
            let next = chain.push_word(word);
            chain.train_choice(state, next);
            state.push(next);
        }
    }
}
