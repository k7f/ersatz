use crate::Reaction;

#[derive(PartialOrd, Ord, PartialEq, Eq, Default, Debug)]
pub struct State(bit_set::BitSet);

impl State {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, elt: usize) {
        self.0.insert(elt);
    }
}

impl Clone for State {
    #[inline]
    fn clone(&self) -> Self {
        State(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.0.clone_from(&other.0)
    }
}

#[derive(Default, Debug)]
pub struct Site {
    reactions: Vec<Reaction>,
    state:     State,
}

impl Site {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_reactions<I>(mut self, reactions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Reaction>,
    {
        self.reactions.extend(reactions.into_iter().map(Into::into));
        self
    }
}
