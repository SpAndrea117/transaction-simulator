use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Deserializer};

use crate::{
    business_logic::{Client, ClientTransaction, Transaction, Type},
    shared::errors::Error,
};

impl From<Type> for String {
    fn from(value: Type) -> Self {
        match value {
            Type::Deposit => "deposit".to_owned(),
            Type::Withdrawal => "withdrawal".to_owned(),
            Type::Dispute => "dispute".to_owned(),
            Type::Resolve => "resolve".to_owned(),
            Type::ChargeBack => "chargeback".to_owned(),
        }
    }
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::ChargeBack),
            _ => Err(Error::InvalidTransactionType(s.to_owned())),
        }
    }
}

pub(super) fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

pub(super) fn four_decimals<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{:.4}", value))
}

// If client transaction is linked to a new client and it is valid (i.e. positive amount on 'deposit' transaction type),
// create new client with correct properties, otherwise, just add new client to tracking list with default properties
impl From<ClientTransaction> for Client {
    fn from(client_transaction: ClientTransaction) -> Self {
        match client_transaction.transaction_type {
            Type::Deposit
                if client_transaction
                    .amount
                    .is_some_and(|amount| amount.is_sign_positive()) =>
            {
                Self {
                    id: client_transaction.id,
                    available: client_transaction.amount.unwrap_or_default(),
                    total: client_transaction.amount.unwrap_or_default(),
                    transations_history: HashMap::from([(
                        client_transaction.tx,
                        Transaction {
                            amount: client_transaction.amount.unwrap_or_default(),
                            is_under_dispute: false,
                        },
                    )]),
                    ..Default::default()
                }
            }
            _ => Self {
                id: client_transaction.id,
                ..Default::default()
            },
        }
    }
}
