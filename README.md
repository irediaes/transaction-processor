# Transaction Processor
A simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

Please note: Further transactions are ignored on frozen(locked) accounts.
### Requirements
* [Download](https://redis.io/download/) and install `Redis`
* Prepare csv transaction file. Check `sample-tx.csv` and `sample-tx-large.csv` for reference.

### Build

Use the following command to build.

`cargo build --release`


### Run

You can run the program without building it by running the command below.

`cargo run -- sample-tx.csv`

or to export the accounts into a file

`cargo run -- sample-tx.csv > accounts.csv`

Or after running `cargo build` command above

`./target/release/transaction-processor sample-tx.csv`


### Tests
Tested all vital units of the system.

To run the unit test;

`cargo test --package transaction-processor --bin transaction-processor`


Expected Results: 
```
test ac::tests::tests::test_account_chargeback ... ok
test ac::tests::tests::test_account_deposit ... ok
test ac::tests::tests::test_account_resolve ... ok
test ac::tests::tests::test_account_withdraw ... ok
test ac::tests::tests::test_account_dispute ... ok
test ac::tests::tests::test_process_deposit ... ok
test ac::tests::tests::test_process_dispute ... ok
test storage::tests::test_storage_insert ... ok
test ac::tests::tests::test_process_chargeback ... ok
test ac::tests::tests::test_process_resolve ... ok
test storage::tests::test_storage_modify ... ok
test ac::tests::tests::test_process_withdrawal ... ok
```


### Upcoming Improvement
* stream values through memory as opposed to loading the entire data set upfront
* add concurrency to the print function

## License

[Apache 2.0](https://choosealicense.com/licenses/apache-2.0/)

