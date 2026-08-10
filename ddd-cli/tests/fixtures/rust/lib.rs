//! M6 fixture crate — the before-state every Rust policy row is judged from.
//!
//! Each row has a triggering neighbour and a non-triggering one already in
//! place, so a test can make the smallest edit that distinguishes them: a
//! `pub` item beside a private one, a public trait beside a private trait, a
//! `pub(crate)` method beside a `pub` one, a public enum beside a private
//! one, a public module beside a private one.

pub mod other;

/// Public trait: members added here force every implementor.
pub trait Greeter {
    fn greet(&self) -> String;
}

/// Private trait: the same edit against this one is not surface.
trait Internal {
    fn tally(&self) -> u32;
}

#[derive(Clone)]
pub struct Person {
    pub name: String,
    age: u32,
}

struct Secret {
    _value: u32,
}

impl Person {
    pub fn new(name: String) -> Self {
        Person { name, age: 0 }
    }

    pub(crate) fn age(&self) -> u32 {
        self.age
    }

    fn bump(&mut self) {
        self.age += 1;
    }
}

impl Greeter for Person {
    fn greet(&self) -> String {
        self.name.clone()
    }
}

impl Internal for Secret {
    fn tally(&self) -> u32 {
        self._value
    }
}

pub enum Colour {
    Red,
    Green,
}

enum Mode {
    _On,
    _Off,
}

pub const LIMIT: u32 = 10;

pub type Count = u32;

pub fn describe(colour: &Colour) -> &'static str {
    match colour {
        Colour::Red => "red",
        Colour::Green => "green",
    }
}

fn internal_helper(value: u32) -> u32 {
    value + 1
}

pub mod visible {
    pub fn exported(a: u32) -> u32 {
        a
    }
}

mod capped {
    pub fn looks_public_is_not(a: u32) -> u32 {
        a
    }
}

// Keep the private items live so the fixture compiles without warnings
// mattering to what is being measured.
impl Person {
    fn touch(&mut self) -> u32 {
        self.bump();
        let _ = internal_helper(capped::looks_public_is_not(1));
        let _ = Mode::_On;
        let s = Secret { _value: 1 };
        s.tally()
    }
}
