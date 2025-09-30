use std::fs; use anyhow::*; pub fn read_all(path:&str)->Result<String>{ Ok(fs::read_to_string(path)?) }
