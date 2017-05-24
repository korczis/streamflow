#[derive(Debug, Clone)]
pub struct Csv {
    pub delimiter: u8,
    pub has_header: bool,
    pub flexible: bool
}
