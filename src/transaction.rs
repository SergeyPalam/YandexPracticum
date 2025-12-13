use super::storage::{Storage, Balance};
use super::errors::BankError;
use std::ops::Add;

pub trait Transaction {
    fn apply(&self, accounts: &mut Storage) -> Result<(), BankError>;
}

pub struct TxCombinator<T1, T2> {
    t1: T1,
    t2: T2,
}

impl<T1, T2, Rhs: Transaction> Add<Rhs> for TxCombinator<T1, T2> {
    type Output = TxCombinator<TxCombinator<T1, T2>, Rhs>;

    fn add(self, rhs: Rhs) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
} 

impl<T1: Transaction, T2: Transaction> Transaction for TxCombinator<T1, T2> {
    fn apply(&self, accounts: &mut Storage) -> Result<(), BankError> {
        self.t1.apply(accounts)?;
        self.t2.apply(accounts)?;
        Ok(())
    }
}

pub struct Deposit {
    account: String,
    amount: Balance,
}

impl Deposit {
    pub fn new(account: &str, amount: Balance) -> Self {
        Self{
            account: account.to_owned(),
            amount,
        }
    }
}

impl Transaction for Deposit {
    fn apply(&self, storage: &mut Storage) -> Result<(), BankError> {
        storage.deposit(&self.account, self.amount)?;
        Ok(())
    }
}

impl<T: Transaction> Add<T> for Deposit {
    type Output = TxCombinator<Deposit, T>;

    fn add(self, rhs: T) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
}

pub struct Withdraw {
    account: String,
    amount: Balance,
}

impl Withdraw {
    pub fn new(account: &str, amount: Balance) -> Self {
        Self{
            account: account.to_owned(),
            amount,
        }
    }
}

impl Transaction for Withdraw {
    fn apply(&self, storage: &mut Storage) -> Result<(), BankError> {
        storage.withdraw(&self.account, self.amount)?;
        Ok(())
    }
}

impl<T: Transaction> Add<T> for Withdraw {
    type Output = TxCombinator<Withdraw, T>;

    fn add(self, rhs: T) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
}

pub struct Transfer {
    from: String,
    to: String,
    amount: Balance,
}

impl Transfer {
    pub fn new(from: &str, to: &str, amount: Balance) -> Self {
        Self {
            from: from.to_owned(),
            to: to.to_owned(),
            amount,
        }
    }
}

impl Transaction for Transfer {
    fn apply(&self, storage: &mut Storage) -> Result<(), BankError> {
        storage.withdraw(&self.from, self.amount)?;
        storage.deposit(&self.to, self.amount)?;
        Ok(())
    }
}

impl<T: Transaction> Add<T> for Transfer {
    type Output = TxCombinator<Transfer, T>;

    fn add(self, rhs: T) -> Self::Output {
        TxCombinator { t1: self, t2: rhs }
    }
} 