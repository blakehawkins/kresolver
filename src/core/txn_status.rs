/// The status of a past transaction. Normally healthy except when ongoing
/// dispute resolution is in-flight.
pub enum TxnStatus {
    Healthy,
    Disputed,
    Skipped,
}
