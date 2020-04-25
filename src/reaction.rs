use crate::{Entity, EntitySet};

#[derive(Clone, Default, Debug)]
pub struct Reaction {
    pub r: EntitySet,
    pub i: EntitySet,
    pub p: EntitySet,
}

impl Reaction {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_reactants<I>(mut self, ents: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Entity>,
    {
        self.p.extend(ents.into_iter().map(Into::into));
        self
    }

    pub fn with_inhibitors<I>(mut self, ents: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Entity>,
    {
        self.p.extend(ents.into_iter().map(Into::into));
        self
    }

    pub fn with_products<I>(mut self, ents: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Entity>,
    {
        self.p.extend(ents.into_iter().map(Into::into));
        self
    }
}
