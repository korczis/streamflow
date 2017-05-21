use time::PreciseTime;

use super::traits::end_of_scope::EndOfScope;

pub struct ProfileObject {
    pub start: PreciseTime
}

impl ProfileObject {
    pub fn new() -> ProfileObject {
        ProfileObject {
            start: PreciseTime::now()
        }
    }
}

impl Default for ProfileObject {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProfileObject {
    fn drop(&mut self) {
        self.end_of_scope()
    }
}

impl EndOfScope for ProfileObject {
    fn end_of_scope(&self) {
        let diff = self.start.to(PreciseTime::now());
        let elapsed_secs = diff.num_nanoseconds().unwrap() as f64 * 1e-9;

        debug!("Drop() - {:?}", elapsed_secs);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
