use std::collections::HashMap;

use bincode::Options as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalNodeDescrption{
    // pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExternalNodeDescrptions{
    pub nodes: HashMap<String, ExternalNodeDescrption>
}

impl ExternalNodeDescrptions{
    pub fn bincode_options()-> impl bincode::config::Options {
        bincode::config::DefaultOptions::new()
            .with_limit(10 * 1024 * 1024)
    }
}