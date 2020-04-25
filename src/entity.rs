use std::{
    collections::{hash_set, HashSet},
    iter::FromIterator,
};
use crate::{Ground, Source, State};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Entity {
    Number(u32),
    Name(String),
    Identifier(String),
}

#[derive(Clone, Default, Debug)]
pub struct EntitySet {
    numbers: HashSet<u32>,
    names:   HashSet<String>,
}

impl EntitySet {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.numbers.clear();
        self.names.clear();
    }

    pub fn extend_numbers<I>(&mut self, nums: I)
    where
        I: IntoIterator,
        I::Item: Into<u32>,
    {
        self.numbers.extend(nums.into_iter().map(Into::into));
    }

    pub fn drain_numbers(&mut self) -> hash_set::Drain<'_, u32> {
        self.numbers.drain()
    }

    pub fn extend_names<I>(&mut self, names: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.names.extend(names.into_iter().map(Into::into));
    }

    pub fn drain_names(&mut self) -> hash_set::Drain<'_, String> {
        self.names.drain()
    }
}

impl Source for EntitySet {
    fn emit(&mut self, ground: &Ground) -> State {
        let mut state = State::new();

        for num in self.numbers.iter() {
            if let Some(id) = ground.get_number_id(num) {
                state.insert(id);
            }
        }

        for name in self.names.iter() {
            if let Some(id) = ground.get_name_id(name) {
                state.insert(id);
            }
        }

        state
    }
}

impl FromIterator<Entity> for EntitySet {
    fn from_iter<I: IntoIterator<Item = Entity>>(ents: I) -> Self {
        let mut entity_set = EntitySet::new();

        entity_set.extend(ents);
        entity_set
    }
}

impl Extend<Entity> for EntitySet {
    fn extend<I>(&mut self, ents: I)
    where
        I: IntoIterator,
        I::Item: Into<Entity>,
    {
        for entity in ents.into_iter().map(Into::into) {
            match entity {
                Entity::Number(n) => {
                    self.numbers.insert(n);
                }
                Entity::Name(n) => {
                    self.names.insert(n);
                }
                Entity::Identifier(id) => {} // FIXME
            }
        }
    }
}
