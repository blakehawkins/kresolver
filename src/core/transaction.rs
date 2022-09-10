use std::str::FromStr;

use crate::amount::Amount;
use crate::record::Record;

/// Intermediate representation of a ledger record. This is either a transaction
/// (deposit/withdrawal) or a dispute-related record (dispute, resolve,
/// chargeback).
pub enum Transaction {
    Deposit(u16, u32, Amount),
    Withdrawal(u16, u32, Amount),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

impl Transaction {
    /// Transform a ledger record from its CSV (string) format to internal
    /// representation.
    pub fn new(record: Record) -> Self {
        match record.ty.as_str() {
            "deposit" => Transaction::Deposit(
                record.client,
                record.tx,
                Amount::from_str(&record.amount).expect("couldn't parse record amount"),
            ),
            "withdrawal" => Transaction::Withdrawal(
                record.client,
                record.tx,
                Amount::from_str(&record.amount).expect("couldn't parse record amount"),
            ),
            "dispute" => Transaction::Dispute(record.client, record.tx),
            "resolve" => Transaction::Resolve(record.client, record.tx),
            "chargeback" => Transaction::Chargeback(record.client, record.tx),
            _ => panic!("Unexpected record type"),
        }
    }
}
