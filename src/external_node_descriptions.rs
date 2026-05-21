use std::collections::HashMap;

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