use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum BankError {
    UserNotFound,
    FundsLimit,
    System(String),
}

impl Display for BankError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BankError::UserNotFound => {
                write!(f, "User not found")
            }
            BankError::FundsLimit => {
                write!(f, "Isn't enought money")
            }
            BankError::System(description) => {
                write!(f, "System error: {description}")
            }
        }
    }
}

impl Error for BankError {}

impl From<std::io::Error> for BankError {
    fn from(value: std::io::Error) -> Self {
        let reason = format!("{value}");
        Self::System(reason)
    }
}

impl From<std::num::ParseIntError> for BankError {
    fn from(value: std::num::ParseIntError) -> Self {
        let reason = format!("{value}");
        Self::System(reason)
    }
}
