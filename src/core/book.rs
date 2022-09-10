use std::collections::BTreeMap;
use std::io::Write;

use anyhow::{Error, Result};

use lru::LruCache;

use crate::account::Account;
use crate::transaction::Transaction;
use crate::txn_state::TxnState;
use crate::Ledger;

/// The cache size for purposes of resolving dispute records. Increasing this value trades off memory usage for
/// correctness.
pub const TXN_CACHE_SIZE: &usize = &1_000_000;

type TransactionCache = LruCache<u32, TxnState>;

/// A representation of accounts state. Can resolve transactions, ledgers, and write accounts state in CSV format.
pub struct Book {
    pub accounts: BTreeMap<u16, Account>,
    pub tx_cache: TransactionCache,
}

impl Book {
    /// A new Book with default transaction-cache size.
    pub fn new() -> Self {
        Book {
            accounts: BTreeMap::new(),
            tx_cache: LruCache::new(*TXN_CACHE_SIZE),
        }
    }

    /// Resolves a single transaction record (including dispute-related records).
    /// Deposits *and* Withdrawals are cached and can be later disputed.
    pub fn resolve(&mut self, transaction: Transaction) -> &mut Self {
        match transaction {
            Transaction::Deposit(acc, txn, amt) => self
                .accounts
                .entry(acc)
                .or_insert_with(|| Account::new(acc))
                .deposit(amt)
                .then(|| {
                    self.tx_cache.put(txn, TxnState::new(acc, amt));
                })
                .unwrap_or_else(|| {
                    self.tx_cache.put(txn, TxnState::skipped(acc, amt));
                }),
            Transaction::Withdrawal(acc, txn, amt) => self
                .accounts
                .entry(acc)
                .or_insert_with(|| Account::new(acc))
                .withdraw(amt)
                .then(|| {
                    self.tx_cache.put(txn, TxnState::new(acc, -amt));
                })
                .unwrap_or_else(|| {
                    self.tx_cache.put(txn, TxnState::skipped(acc, -amt));
                }),
            Transaction::Dispute(acc, txn) => self
                .accounts
                .entry(acc)
                .or_insert_with(|| Account::new(acc))
                .dispute(self.tx_cache.get_mut(&txn))
                .then(|| ())
                .unwrap_or(()),
            Transaction::Resolve(acc, txn) => self
                .accounts
                .entry(acc)
                .or_insert_with(|| Account::new(acc))
                .resolve(self.tx_cache.get_mut(&txn))
                .then(|| ())
                .unwrap_or(()),
            Transaction::Chargeback(acc, txn) => self
                .accounts
                .entry(acc)
                .or_insert_with(|| Account::new(acc))
                .chargeback(self.tx_cache.get_mut(&txn))
                .then(|| ())
                .unwrap_or(()),
        };

        self
    }

    /// Resolves all the transactions from the provided ledger.
    pub fn resolve_all<T: Into<Ledger>>(&mut self, ledger: T) -> &mut Self {
        ledger.into().for_each(|txn| {
            self.resolve(txn);
        });
        self
    }

    /// Resolves all the transactions from the provided ledger. Fails if the
    /// ledger is invalid.
    pub fn try_resolve_all<T: TryInto<Ledger>>(&mut self, ledger: T) -> Result<&mut Self>
    where
        anyhow::Error: From<T::Error>,
    {
        ledger.try_into()?.for_each(|txn| {
            self.resolve(txn);
        });

        Ok(self)
    }

    /// Write the internal account state as a CSV to the provided writer.
    pub fn write<T: Write>(&self, write: T) -> Result<&Self, Error> {
        let mut writer = csv::WriterBuilder::new().from_writer(write);
        self.accounts
            .iter()
            .try_for_each(|(_, acc)| writer.serialize(acc))?;

        writer.flush()?;

        Ok(self)
    }
}

impl Default for Book {
    fn default() -> Self {
        Self::new()
    }
}
