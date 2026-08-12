//! Corpus fixture crate — the base state each labelled case edits.

/// Public trait: members added here force every implementor.
pub trait Greeter {
    fn greet(&self) -> String;
}

#[derive(Clone)]
pub struct Person {
    pub name: String,
    age: u32,
}

impl Person {
    pub fn new(name: String) -> Self {
        Person { name, age: 0 }
    }

    pub(crate) fn age(&self) -> u32 {
        self.age
    }
}

impl Greeter for Person {
    fn greet(&self) -> String {
        self.name.clone()
    }
}

pub struct Widget {
    pub id: u32,
}

pub enum Colour {
    Red,
    Green,
}

fn internal_helper(value: u32) -> u32 {
    value + 1
}
