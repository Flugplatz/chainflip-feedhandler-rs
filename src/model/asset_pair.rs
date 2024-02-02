use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AssetPair {
    pub from: String,
    pub to: String,
}

impl AssetPair {
    pub fn new(from: String, to: String) -> AssetPair {
        AssetPair { from, to }
    }
}

impl fmt::Display for AssetPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.from, self.to)
    }
}

impl fmt::Debug for AssetPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
