use std::{
    collections::{hash_map, HashMap},
    fs,
    path::Path,
    error::Error,
};
use crate::{Entity, Site, State};

#[derive(Default, Debug)]
pub struct Ersatz {
    ground:    Ground,
    sites:     Vec<Site>,
    max_steps: Option<usize>,
}

impl Ersatz {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_max_steps(&mut self, val: usize) {
        self.max_steps = Some(val);
    }

    pub fn with_sites<I>(mut self, sites: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Site>,
    {
        self.sites.extend(sites.into_iter().map(Into::into));
        self
    }

    pub fn merge(&mut self, other: Self) {
        if self.max_steps.is_none() {
            self.max_steps = other.max_steps;
        }

        self.ground.merge(other.ground);
        self.sites.extend(other.sites);
    }

    pub fn add_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let spec = fs::read_to_string(path.as_ref())?;
        let other: Ersatz = spec.parse()?;

        self.merge(other);

        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut ersatz = Ersatz::new();

        ersatz.add_from_file(path)?;

        Ok(ersatz)
    }
}

#[derive(Default, Debug)]
pub struct Ground {
    entities:   Vec<Entity>,
    number_ids: HashMap<u32, usize>,
    name_ids:   HashMap<String, usize>,
}

impl Ground {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, entity: Entity) -> bool {
        match entity {
            Entity::Number(num) => self.insert_number(num),
            Entity::Name(name) => self.insert_name(name),
            Entity::Identifier(_) => false,
        }
    }

    pub fn insert_number(&mut self, num: u32) -> bool {
        match self.number_ids.entry(num) {
            hash_map::Entry::Occupied(_) => false,
            hash_map::Entry::Vacant(e) => {
                e.insert(self.entities.len());
                self.entities.push(Entity::Number(num));

                true
            }
        }
    }

    pub fn insert_name(&mut self, name: String) -> bool {
        match self.name_ids.entry(name) {
            hash_map::Entry::Occupied(_) => false,
            hash_map::Entry::Vacant(e) => {
                let name = e.key().clone();

                e.insert(self.entities.len());
                self.entities.push(Entity::Name(name));

                true
            }
        }
    }

    #[inline]
    pub fn get_id(&self, entity: &Entity) -> Option<usize> {
        match entity {
            Entity::Number(num) => self.get_number_id(num),
            Entity::Name(name) => self.get_name_id(&name),
            Entity::Identifier(_) => None,
        }
    }

    #[inline]
    pub fn get_number_id(&self, num: &u32) -> Option<usize> {
        self.number_ids.get(num).copied()
    }

    pub fn provide_number_id(&mut self, num: &u32) -> usize {
        match self.number_ids.entry(*num) {
            hash_map::Entry::Occupied(e) => *e.get(),
            hash_map::Entry::Vacant(e) => {
                let id = self.entities.len();

                e.insert(id);
                self.entities.push(Entity::Number(*num));

                id
            }
        }
    }

    #[inline]
    pub fn get_name_id<S: AsRef<str>>(&self, name: S) -> Option<usize> {
        self.name_ids.get(name.as_ref()).copied()
    }

    pub fn provide_name_id<S: AsRef<str> + Copy>(&mut self, name: S) -> usize {
        self.get_name_id(name).unwrap_or_else(|| {
            let name = name.as_ref().to_string();
            let id = self.entities.len();

            self.entities.push(Entity::Name(name.clone()));
            self.name_ids.insert(name, id);

            id
        })
    }

    pub fn merge(&mut self, mut other: Self) {
        for (num, _) in other.number_ids.drain() {
            self.insert_number(num);
        }

        for (name, _) in other.name_ids.drain() {
            self.insert_name(name);
        }
    }
}

pub trait Source {
    fn emit(&mut self, ground: &Ground) -> State;
}
