use super::File;

pub struct Options {

}

#[derive(Default)]
pub struct Csv {
}

impl Csv {
    pub fn new() -> Csv {
        Csv {

        }
    }
}

impl File for Csv {
    fn from_path() {}
}

