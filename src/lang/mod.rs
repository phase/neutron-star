// language items for both the ast and ir

pub mod refcap;
pub mod ptr;
pub mod primitive;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<String>);

pub static SEPARATOR: &str = "::";

impl Path {
    pub fn new() -> Self {
        Path(vec![])
    }

    pub fn of(s: &str) -> Self {
        Self(vec![s.to_string()])
    }

    pub fn append(&self, s: String) -> Self {
        let mut vec = self.0.clone();
        vec.push(s);
        Self(vec)
    }

    pub fn pop(&mut self) -> String {
        self.0.pop().expect("tried to pop empty path")
    }

    pub fn to_string(&self) -> String {
        self.0.join(SEPARATOR)
    }
}

