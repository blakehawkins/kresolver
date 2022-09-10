use crate::amount::Amount;
use crate::txn_status::TxnStatus;

/// Cached state of a past transaction.
pub struct TxnState {
    pub account: u16,
    pub amount: Amount,
    pub status: TxnStatus,
}

impl TxnState {
    /// A new, healthy transaction.
    pub fn new(account: u16, amount: Amount) -> Self {
        TxnState {
            account,
            amount,
            status: TxnStatus::Healthy,
        }
    }

    /// A transaction that was skipped because at the point that it was to be
    /// processed, validity checks failed.
    pub fn skipped(account: u16, amount: Amount) -> Self {
        TxnState {
            account,
            amount,
            status: TxnStatus::Skipped,
        }
    }
}
