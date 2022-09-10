use serde::Serialize;

use crate::amount::Amount;
use crate::txn_state::TxnState;
use crate::txn_status::TxnStatus;

/// An individual's account, unique by `id`.
#[derive(Default, Serialize)]
pub struct Account {
    pub id: u16,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl Account {
    /// Creates a new/empty account with the unique ID provided.
    pub fn new(id: u16) -> Self {
        Account {
            id,
            locked: false,
            ..Default::default()
        }
    }

    /// Fails if the account is locked.
    pub fn deposit(&mut self, amount: Amount) -> bool {
        if self.locked {
            return false;
        }

        self.available += amount;
        self.total += amount;

        true
    }

    /// Fails if the account is locked or if available funds are too low.
    pub fn withdraw(&mut self, amount: Amount) -> bool {
        if self.locked || self.available < amount {
            return false;
        }

        self.available -= amount;
        self.total -= amount;
        true
    }

    /// Marks a transaction as disputed. If the transaction is too old, the
    /// account is already locked, the transaction was never applied, or the
    /// transaction is already disputed, then this will have no effect.
    pub fn dispute(&mut self, txn: Option<&mut TxnState>) -> bool {
        if self.locked {
            return false;
        }

        match txn {
            None => false,
            Some(state) => {
                assert_eq!(self.id, state.account);

                match state.status {
                    TxnStatus::Healthy => {
                        state.status = TxnStatus::Disputed;
                        self.available -= state.amount;
                        self.held += state.amount;
                        true
                    }
                    TxnStatus::Disputed => false, // Transaction is already disputed.
                    TxnStatus::Skipped => false, // Transaction was never applied because it was invalid.
                }
            }
        }
    }

    /// Marks a transaction as resolved. If the transaction is too old, the
    /// account is locked, the transaction was never applied, or the
    /// transaction is not currently disputed, then this will have no effect.
    pub fn resolve(&mut self, txn: Option<&mut TxnState>) -> bool {
        if self.locked {
            return false;
        }

        match txn {
            None => false,
            Some(state) => {
                assert_eq!(self.id, state.account);

                match state.status {
                    TxnStatus::Healthy => false, // Transaction is not currently disputed.
                    TxnStatus::Disputed => {
                        state.status = TxnStatus::Healthy;
                        self.available += state.amount;
                        self.held -= state.amount;
                        true
                    }
                    TxnStatus::Skipped => false, // Transaction was never applied because it was invalid.
                }
            }
        }
    }

    /// Marks a transaction as charged back. If the transaction is too old, the
    /// account is locked, the transaction was never applied, or the
    /// transaction is not currently disputed, then this will have no effect.
    /// Otherwise, the account is locked.
    pub fn chargeback(&mut self, txn: Option<&mut TxnState>) -> bool {
        if self.locked {
            return false;
        }

        match txn {
            None => false,
            Some(state) => {
                assert_eq!(self.id, state.account);

                match state.status {
                    TxnStatus::Healthy => false, // Transaction is not currently disputed.
                    TxnStatus::Disputed => {
                        self.locked = true;
                        self.total -= state.amount;
                        self.held -= state.amount;
                        true
                    }
                    TxnStatus::Skipped => false, // Transaction was never applied because it was invalid.
                }
            }
        }
    }
}
