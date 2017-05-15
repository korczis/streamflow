use super::producer::Producer;

pub trait Source {
    // fn next() {}
}

impl Producer for Source {
    fn produce() {}
}

pub mod file;
