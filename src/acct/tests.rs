use crate::acct::account::{self, Account};
use crate::acct::storage;
use crate::tx::{storage as TxStore, transaction::Transaction};

/// Tests

#[test]
fn test_process_deposit() {
    let tranx_1 = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 10.0,
    };

    let tranx_2 = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 11,
        amount: 15.0,
    };

    account::process_deposit(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_1.client, |acc| acc.unwrap().clone());

        let tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx_1.tx, |tranx| tranx.unwrap().clone());

        assert!(
            acct.available == tranx_1.amount,
            "invalid available funds; expected {}, got {}",
            tranx_1.amount,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.total
        );

        assert!(
            acct.total == tranx_1.amount,
            "invalid total funds; expected {}, got {}",
            tranx_1.amount,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        assert!(
            tranx.r#type == tranx_1.r#type,
            "invalid transaction type funds; expected {}, got {}",
            tranx_1.r#type,
            tranx.r#type
        );

        assert!(
            tranx.amount == tranx_1.amount,
            "invalid transaction amount funds; expected {}, got {}",
            tranx_1.amount,
            tranx.amount
        );

        assert!(
            tranx.client == tranx_1.client,
            "invalid transaction client funds; expected {}, got {}",
            tranx_1.client,
            tranx.client
        );
    }

    account::process_deposit(&tranx_2);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_2.client, |acc| acc.unwrap().clone());

        let tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx_2.tx, |tranx| tranx.unwrap().clone());

        let available = tranx_1.amount + tranx_2.amount;

        assert!(
            acct.available == available,
            "invalid available funds; expected {}, got {}",
            available,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.total
        );

        assert!(
            acct.total == available,
            "invalid total funds; expected {}, got {}",
            available,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        assert!(
            tranx.r#type == tranx_2.r#type,
            "invalid transaction type funds; expected {}, got {}",
            tranx_2.r#type,
            tranx.r#type
        );

        assert!(
            tranx.amount == tranx_2.amount,
            "invalid transaction amount funds; expected {}, got {}",
            tranx_2.amount,
            tranx.amount
        );

        assert!(
            tranx.client == tranx_2.client,
            "invalid transaction client funds; expected {}, got {}",
            tranx_2.client,
            tranx.client
        );
    }
}

#[test]
fn test_process_withdrawal() {
    let client = 2;
    let tranx_withdrawal = Transaction {
        r#type: "withdrawal".to_string(),
        client,
        tx: 2,
        amount: 10.0,
    };

    let tranx_withdrawal_2 = Transaction {
        r#type: "withdrawal".to_string(),
        client,
        tx: 22,
        amount: 10.0,
    };

    let tranx_deposit = Transaction {
        r#type: "deposit".to_string(),
        client,
        tx: 222,
        amount: 15.0,
    };

    account::process_withdrawal(&tranx_withdrawal);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == 0.0,
            "invalid available funds; expected {}, got {}",
            0.0,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.total
        );

        assert!(
            acct.total == 0.0,
            "invalid total funds; expected {}, got {}",
            0.0,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        let tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx_withdrawal.tx, |tranx| tranx.unwrap().clone());

        assert!(
            tranx.r#type == tranx_withdrawal.r#type,
            "invalid transaction type funds; expected {}, got {}",
            tranx_withdrawal.r#type,
            tranx.r#type
        );

        assert!(
            tranx.amount == tranx_withdrawal.amount,
            "invalid transaction amount funds; expected {}, got {}",
            tranx_withdrawal.amount,
            tranx.amount
        );

        assert!(
            tranx.client == tranx_withdrawal.client,
            "invalid transaction client funds; expected {}, got {}",
            tranx_withdrawal.client,
            tranx.client
        );
    }

    account::process_deposit(&tranx_deposit);
    account::process_withdrawal(&tranx_withdrawal_2);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(client, |acc| acc.unwrap().clone());
        let amount_diff = tranx_deposit.amount - tranx_withdrawal_2.amount;
        assert!(
            acct.available == amount_diff,
            "invalid available funds; expected {}, got {}",
            amount_diff,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.total
        );

        assert!(
            acct.total == amount_diff,
            "invalid total funds; expected {}, got {}",
            amount_diff,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );
    }
}

#[test]
fn test_process_dispute() {
    let tranx_dispute = Transaction::new("dispute".to_string(), 3, 33, 0.0);

    let tranx_deposit = Transaction::new("deposit".to_string(), 3, 3, 15.0);
    let tranx_deposit_2 = Transaction::new("deposit".to_string(), 3, 33, 10.0);

    account::process_dispute(&tranx_dispute);

    unsafe {
        storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| {
                assert!(
                    acc == None,
                    "invalid available funds; expected {}, got {:?}",
                    "None",
                    acc
                );
            });
    }

    account::process_deposit(&tranx_deposit);
    account::process_deposit(&tranx_deposit_2);
    account::process_dispute(&tranx_dispute);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| acc.unwrap().clone());
        assert!(
            acct.available == tranx_deposit.amount,
            "invalid available funds; expected {}, got {}",
            tranx_deposit.amount,
            acct.available
        );

        assert!(
            acct.held == tranx_deposit_2.amount,
            "invalid held funds; expected {}, got {}",
            tranx_deposit_2.amount,
            acct.held
        );

        let total = tranx_deposit.amount + tranx_deposit_2.amount;
        assert!(
            acct.total == total,
            "invalid total funds; expected {}, got {}",
            total,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        let dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx_dispute.tx, |acc| acc.unwrap().clone());

        assert!(
            dispute.tx == tranx_dispute.tx,
            "invalid dispute tx; expected {}, got {}",
            tranx_dispute.tx,
            dispute.tx
        );

        assert!(
            dispute.client == tranx_dispute.client,
            "invalid dispute client; expected {}, got {}",
            tranx_dispute.client,
            dispute.client
        );
    }
}

#[test]
fn test_process_resolve() {
    let tranx_dispute = Transaction::new("dispute".to_string(), 4, 44, 0.0);
    let tranx_resolve = Transaction::new("resolve".to_string(), 4, 44, 0.0);

    let tranx_deposit = Transaction::new("deposit".to_string(), 4, 4, 15.0);
    let tranx_deposit_2 = Transaction::new("deposit".to_string(), 4, 44, 10.0);

    account::process_resolve(&tranx_resolve);

    unsafe {
        storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| {
                assert!(
                    acc == None,
                    "invalid available funds; expected {}, got {:?}",
                    "None",
                    acc
                );
            });
    }

    account::process_deposit(&tranx_deposit);
    account::process_deposit(&tranx_deposit_2);
    account::process_dispute(&tranx_dispute);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| acc.unwrap().clone());
        assert!(
            acct.available == tranx_deposit.amount,
            "invalid available funds; expected {}, got {}",
            tranx_deposit.amount,
            acct.available
        );

        assert!(
            acct.held == tranx_deposit_2.amount,
            "invalid held funds; expected {}, got {}",
            tranx_deposit_2.amount,
            acct.held
        );

        let total = tranx_deposit.amount + tranx_deposit_2.amount;
        assert!(
            acct.total == total,
            "invalid total funds; expected {}, got {}",
            total,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        let dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx_dispute.tx, |acc| acc.unwrap().clone());

        assert!(
            dispute.tx == tranx_dispute.tx,
            "invalid dispute tx; expected {}, got {}",
            tranx_dispute.tx,
            dispute.tx
        );

        assert!(
            dispute.client == tranx_dispute.client,
            "invalid dispute client; expected {}, got {}",
            tranx_dispute.client,
            dispute.client
        );

        assert!(
            dispute.resolved == false,
            "invalid dispute client; expected {}, got {}",
            false,
            dispute.resolved
        );
    }
    // test resolve
    account::process_resolve(&tranx_resolve);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| acc.unwrap().clone());

        let available = tranx_deposit.amount + tranx_deposit_2.amount;
        assert!(
            acct.available == available,
            "invalid available funds; expected {}, got {}",
            available,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.held
        );

        let total = tranx_deposit.amount + tranx_deposit_2.amount;
        assert!(
            acct.total == total,
            "invalid total funds; expected {}, got {}",
            total,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        let dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx_dispute.tx, |acc| acc.unwrap().clone());

        assert!(
            dispute.tx == tranx_dispute.tx,
            "invalid dispute tx; expected {}, got {}",
            tranx_dispute.tx,
            dispute.tx
        );

        assert!(
            dispute.client == tranx_dispute.client,
            "invalid dispute client; expected {}, got {}",
            tranx_dispute.client,
            dispute.client
        );

        assert!(
            dispute.resolved == true,
            "invalid dispute client; expected {}, got {}",
            true,
            dispute.resolved
        );
    }
}

#[test]
fn test_process_chargeback() {
    let tranx_dispute = Transaction::new("dispute".to_string(), 5, 55, 0.0);
    let tranx_chargeback = Transaction::new("chargeback".to_string(), 5, 55, 0.0);

    let tranx_deposit = Transaction::new("deposit".to_string(), 5, 5, 15.0);
    let tranx_deposit_2 = Transaction::new("deposit".to_string(), 5, 55, 10.0);

    // test not existing dispute
    account::process_chargeback(&tranx_chargeback);

    unsafe {
        storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| {
                assert!(
                    acc == None,
                    "invalid available funds; expected {}, got {:?}",
                    "None",
                    acc
                );
            });
    }

    account::process_deposit(&tranx_deposit);
    account::process_deposit(&tranx_deposit_2);
    account::process_dispute(&tranx_dispute);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| acc.unwrap().clone());
        assert!(
            acct.available == tranx_deposit.amount,
            "invalid available funds; expected {}, got {}",
            tranx_deposit.amount,
            acct.available
        );

        assert!(
            acct.held == tranx_deposit_2.amount,
            "invalid held funds; expected {}, got {}",
            tranx_deposit_2.amount,
            acct.held
        );

        let total = tranx_deposit.amount + tranx_deposit_2.amount;
        assert!(
            acct.total == total,
            "invalid total funds; expected {}, got {}",
            total,
            acct.total
        );

        assert!(
            acct.locked == false,
            "wrong locked status; expect {}, got {}",
            false,
            acct.locked
        );

        let dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx_dispute.tx, |acc| acc.unwrap().clone());

        assert!(
            dispute.tx == tranx_dispute.tx,
            "invalid dispute tx; expected {}, got {}",
            tranx_dispute.tx,
            dispute.tx
        );

        assert!(
            dispute.client == tranx_dispute.client,
            "invalid dispute client; expected {}, got {}",
            tranx_dispute.client,
            dispute.client
        );

        assert!(
            dispute.resolved == false,
            "invalid dispute client; expected {}, got {}",
            false,
            dispute.resolved
        );
    }
    // test chargeback
    account::process_chargeback(&tranx_chargeback);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_dispute.client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == tranx_deposit.amount,
            "invalid available funds; expected {}, got {}",
            tranx_deposit.amount,
            acct.available
        );

        assert!(
            acct.held == 0.0,
            "invalid held funds; expected {}, got {}",
            0.0,
            acct.held
        );

        assert!(
            acct.total == tranx_deposit.amount,
            "invalid total funds; expected {}, got {}",
            tranx_deposit_2.amount,
            acct.total
        );

        assert!(
            acct.locked == true,
            "wrong locked status; expect {}, got {}",
            true,
            acct.locked
        );

        let dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx_dispute.tx, |acc| acc.unwrap().clone());

        assert!(
            dispute.tx == tranx_dispute.tx,
            "invalid dispute tx; expected {}, got {}",
            tranx_dispute.tx,
            dispute.tx
        );

        assert!(
            dispute.client == tranx_dispute.client,
            "invalid dispute client; expected {}, got {}",
            tranx_dispute.client,
            dispute.client
        );

        assert!(
            dispute.resolved == true,
            "invalid dispute client; expected {}, got {}",
            true,
            dispute.resolved
        );
    }
}

#[test]
fn test_account_deposit() {
    let mut account = Account::new(1, 20.0, 0.0);
    let tranx = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    // Test initial funds
    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    account.deposit(&tranx);

    // Test deposited funds
    assert!(
        account.available == 35.0,
        "wrong available funds; expect {}, got {}",
        35.0,
        account.available
    );

    assert!(
        account.total == 35.0,
        "wrong total funds; expect {}, got {}",
        35.0,
        account.total
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );
}

#[test]
fn test_account_withdraw() {
    let mut account = Account::new(1, 20.0, 0.0);
    let mut tranx = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    // Test initial funds
    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test withdrawing excess funds

    tranx.amount = 50.0;

    account.withdraw(&tranx);

    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test withdrawn funds

    tranx.amount = 5.0;
    account.withdraw(&tranx);

    assert!(
        account.available == 15.0,
        "wrong available funds; expect {}, got {}",
        15.0,
        account.available
    );

    assert!(
        account.total == 15.0,
        "wrong total funds; expect {}, got {}",
        15.0,
        account.total
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );
}

#[test]
fn test_account_dispute() {
    let mut account = Account::new(1, 20.0, 0.0);
    let mut tranx = Transaction {
        r#type: "dispute".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    // Test initial funds
    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test disputing excess funds

    tranx.amount = 50.0;

    account.dispute(&tranx);

    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test disputing funds

    tranx.amount = 15.0;
    account.dispute(&tranx);

    assert!(
        account.available == 5.0,
        "wrong available funds; expect {}, got {}",
        5.0,
        account.available
    );

    assert!(
        account.held == 15.0,
        "wrong held funds; expect {}, got {}",
        15.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );
}

#[test]
fn test_account_resolve() {
    let mut account = Account::new(1, 0.0, 20.0);
    let mut tranx_deposit = Transaction::new("deposit".to_string(), 1, 1, 20.0);

    // Test initial funds
    assert!(
        account.available == 0.0,
        "wrong available funds; expect {}, got {}",
        0.0,
        account.available
    );

    assert!(
        account.held == 20.0,
        "wrong held funds; expect {}, got {}",
        20.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test resolving excess funds

    tranx_deposit.amount = 50.0;

    account.resolve(&tranx_deposit);

    assert!(
        account.available == 0.0,
        "wrong available funds; expect {}, got {}",
        0.0,
        account.available
    );

    assert!(
        account.held == 20.0,
        "wrong held funds; expect {}, got {}",
        20.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test resolving funds

    tranx_deposit.amount = 20.0;
    account.resolve(&tranx_deposit);

    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );
}

#[test]
fn test_account_chargeback() {
    let mut account = Account::new(1, 0.0, 20.0);
    let mut tranx_deposit = Transaction::new("deposit".to_string(), 1, 1, 20.0);

    // Test initial funds
    assert!(
        account.available == 0.0,
        "wrong available funds; expect {}, got {}",
        0.0,
        account.available
    );

    assert!(
        account.held == 20.0,
        "wrong held funds; expect {}, got {}",
        20.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test resolving excess funds

    tranx_deposit.amount = 50.0;

    account.chargeback(&tranx_deposit);

    assert!(
        account.available == 0.0,
        "wrong available funds; expect {}, got {}",
        0.0,
        account.available
    );

    assert!(
        account.held == 20.0,
        "wrong held funds; expect {}, got {}",
        20.0,
        account.held
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    assert!(
        account.locked == false,
        "wrong locked status; expect {}, got {}",
        false,
        account.locked
    );

    // Test resolving funds

    tranx_deposit.amount = 20.0;
    account.chargeback(&tranx_deposit);

    assert!(
        account.available == 0.0,
        "wrong available funds; expect {}, got {}",
        0.0,
        account.available
    );

    assert!(
        account.held == 0.0,
        "wrong held funds; expect {}, got {}",
        0.0,
        account.held
    );

    assert!(
        account.total == 0.0,
        "wrong total funds; expect {}, got {}",
        0.0,
        account.total
    );

    assert!(
        account.locked == true,
        "wrong locked status; expect {}, got {}",
        true,
        account.locked
    );
}
