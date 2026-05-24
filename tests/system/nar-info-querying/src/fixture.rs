pub struct PopulatedFile {
    pub hash: String,
}

pub struct TestFixtures {
    populated: Vec<PopulatedFile>,
}

pub const INVALID_HASH: &str = "00000000000000000000000000000000";

impl TestFixtures {
    pub fn new() -> Self {
        Self {
            populated: Vec::new(),
        }
    }

    pub fn populated(&self) -> &[PopulatedFile] {
        &self.populated
    }

    pub fn add_populated(&mut self, hash: String) {
        self.populated.push(PopulatedFile { hash });
    }

    pub fn valid_hashes(&self) -> Vec<&str> {
        self.populated.iter().map(|p| p.hash.as_str()).collect()
    }
}
