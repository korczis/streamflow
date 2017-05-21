#[derive(Debug, Default, Clone)]
pub struct OptionsCsv {
    pub delimiter: u8,
    pub has_header: bool,
    pub flexible: bool
}
