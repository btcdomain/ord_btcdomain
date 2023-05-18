use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InscribeContent {
    pub content: Vec<u8>,
    pub inscribe_num: u64,
    pub inscribe_id: String,
    pub timestamp: u32,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct InscribeContentSimple {
    pub content: Vec<u8>,
    pub inscribe_num: u64,
    pub inscribe_id: String,
    pub timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InscribeBrc20Content {
    pub content: Vec<u8>,
    pub content_type: String,
    pub inscribe_id: String,
    pub inscribe_num: u64,
    pub timestamp: u32,
    pub output_address: String,
    pub input_address: String,
    pub first_owner: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct InscriptionTotal {
    pub total: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct InscriptionFirstOwner {
    pub first_owner: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct InscriptionContentType {
    pub content_type: String,
    pub address: String,
}