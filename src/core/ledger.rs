use std::io::Cursor;
use std::path::PathBuf;

use anyhow::{Error, Result};
use csv::Reader;

use crate::record::Record;
use crate::transaction::Transaction;

/// An iterator over transactions. Construct using `TryFrom<Path>` or `TryFrom<String>`.
pub struct Ledger(Box<dyn Iterator<Item = Transaction>>);

impl Iterator for Ledger {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn ledger_from_csv_reader<T>(reader: Reader<T>) -> Ledger
where
    T: std::io::Read + 'static,
{
    let transaction_iter = reader
        .into_deserialize()
        .map(|v| {
            let record: Result<Record, csv::Error> = v;
            record.expect("Undeserialisable record")
        })
        .map(Transaction::new);
    Ledger(Box::new(transaction_iter))
}

impl TryFrom<PathBuf> for Ledger {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Error> {
        let reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(path)?;
        Ok(ledger_from_csv_reader(reader))
    }
}

impl TryFrom<String> for Ledger {
    type Error = Error;

    fn try_from(st: String) -> Result<Self, Error> {
        let reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(Cursor::new(st));

        Ok(ledger_from_csv_reader(reader))
    }
}
