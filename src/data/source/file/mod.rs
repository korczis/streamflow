pub trait File {
    fn from_path();
}

pub mod csv;

pub use self::csv::*;

