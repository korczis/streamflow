#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate time;

pub mod data;
pub mod options;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
