Time spent: 3 evenings (about 6 hours)

kresolver runs as a one-shot CLI, and internally implements an API for resolving ledgers. The implementation is
in-memory, collect-free, clone-free, and streaming I/O on both the read and write steps. The top-level of the API is
the `Book`, which maintains state of accounts and has a fixed-size LRU cache for references to past transactions.

Regarding this LRU cache, a decision was made w.r.t. balancing (1) transaction amount accuracy, (2) keeping a perfect
record of past transactions, and (3) performance -- kresolver assumes that progress on any disputed transaction will be
made _before_ 1Mn _other_ records are processed. Otherwise, dispute-related records may be skipped as invalid. This
design optimises for performance and accuracy of transaction amounts.

In hindsight, using big-decimals for transaction amounts is overkill - the all-time total volume of all ethereum
transactions could fit into one such big-decimal number. On the other hand, using a smaller representation would not
solve the problem of dealing with billions of transaction IDs in-memory.

## Extension

Note for clarity: nothing in _this readme section_ is implemented.

Further to the current implementation discussed above, the design could be extended in a number of ways:

1. The cache miss behavior could trigger a read from disk as backup storage for the transaction cache. sqlite would
be a good choice for this.
2. (easier) One could just use `swap` and increase the cache size to 5Bn, which would negatively impact performance but
easily fit all possible 32bit transactions onto a single disk (~70GB).
3. (faster) A creative use of encoding could compress the transaction cache to hold it in physical memory. For example,
you could periodically shift out blocks of 1000 transactions from the cache into a secondary memory storage where each
block is brotli-compressed.

Further, if we relax the specification a little by only guaranteeing in-order processing of deposits and withdrawals, we
could instead split the dispute resolution and transaction storage layer into separate services, and then process
dispute-related records asynchronously. This would be more elegant, faster in mainline usage, and decompose into a more
maintainable architecture -- but the trade-off would be to break correctness (per the specification) due to the
out-of-order processing of dispute resolution steps.

## Notes on safety and correctness

1. I use unit tests to check the main use-cases.
2. I use the type system to balance correctness, performance, error-handling, and API elegance rather than optimising
just one of these.
   - e.g.: I prefer to exit if there's a malformed CSV record rather than risk parsing the CSV incorrectly.
   - e.g.: the `Book` API exposes a `write<T: Write>(write: T)` instead of a `collect()` or `to_vec()` method. The minor
     inconvenience of managing a mut buffer in test cases affords better performance and flexibility for writing large
     books.
   - e.g.: there's an assert to ensure that the account specified in dispute-related records matches the account on the
     disputed transaction -- the application will panic if you try to dispute a transaction on behalf of an innocent
     account.
3. The main shortcoming w.r.t. correctness is inability to handle dispute resolution for very non-recent transactions. I
explain this further above.

## A note about running this in a concurrent server

If you wanted to run this as a long-lived service with active TCP connections, some new challenges/goals are surfaced:

1. It's no longer desirable to panic, barring extreme circumstances - instead, it would be better to fail individual
ledger resolutions and "400" the problematic requests.
2. If it's desireable to merge separate ledgers and account states, you'd need to internally queue requests or block
concurrent writes on a shared storage layer. You'd also want to carefully ensure that transactionality is achieved if
it's possible to panic or fail a ledger resolution.
3. You'd want observability -- starting with metrics and event logs.
4. You'd want to QoS and reconnect clients as needed.

## Performance

On my WSL VM with SSD, kresolver processes about 500k records per second.

```
$ hyperfine './target/release/kresolver tests/350k_transactions.csv'
Benchmark 1: ./target/release/kresolver tests/350k_transactions.csv
  Time (mean ± σ):     689.6 ms ±  24.0 ms    [User: 308.0 ms, System: 50.4 ms]
  Range (min … max):   657.5 ms … 727.7 ms    10 runs
```