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
        tx: 1,
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

        assert!(
            acct.available == (tranx_1.amount + tranx_2.amount),
            "invalid available funds; expected {}, got {}",
            tranx_1.amount,
            acct.available
        );
    }
}

#[test]
fn test_process_withdrawal() {
    let tranx_1 = Transaction {
        r#type: "withdrawal".to_string(),
        client: 2,
        tx: 2,
        amount: 10.0,
    };

    let tranx_2 = Transaction {
        r#type: "deposit".to_string(),
        client: 2,
        tx: 2,
        amount: 15.0,
    };

    account::process_withdrawal(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_1.client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == 0.0,
            "invalid available funds; expected {}, got {}",
            0.0,
            acct.available
        );

        let tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx_1.tx, |tranx| tranx.unwrap().clone());

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
    account::process_withdrawal(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read(tranx_2.client, |acc| acc.unwrap().clone());
        let amount_diff = tranx_2.amount - tranx_1.amount;
        assert!(
            acct.available == amount_diff,
            "invalid available funds; expected {}, got {}",
            amount_diff,
            acct.available
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
fn test_deposit() {
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
}

#[test]
fn test_withdraw() {
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
}

#[test]
fn test_dispute() {
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
}

#[test]
fn test_resolve() {
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
}
