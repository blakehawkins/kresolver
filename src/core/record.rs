use serde::Deserialize;

/// Deserialised row from a ledger CSV.
#[derive(Deserialize)]
pub struct Record {
    #[serde(rename = "type")]
    pub ty: String,
    pub client: u16,
    pub tx: u32,
    pub amount: String,
}
