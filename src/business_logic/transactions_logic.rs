use crate::business_logic::{Client, ClientTransaction, Transaction, Type};

impl Client {
    pub(super) fn apply_transaction(&mut self, transaction: &ClientTransaction) {
        if transaction
            .amount
            .is_some_and(|amount| amount.is_sign_negative())
            || self.locked
        {
            // Ignore invalid transactions and transactions on locked client
            return;
        }

        match transaction.transaction_type {
            Type::Deposit if !self.transations_history.contains_key(&transaction.tx) => {
                if let Some(amount) = transaction.amount {
                    self.available += amount;
                    self.total += amount;
                    self.transations_history.insert(
                        transaction.tx,
                        Transaction {
                            amount: amount,
                            is_under_dispute: false,
                        },
                    );
                }
            }
            Type::Withdrawal if !self.transations_history.contains_key(&transaction.tx) => {
                if let Some(amount) = transaction.amount {
                    if self.available < amount {
                        return; // ignore withdrawal if funds are not sufficient
                    }
                    self.available -= amount;
                    self.total -= amount;
                    self.transations_history.insert(
                        transaction.tx,
                        Transaction {
                            amount: amount,
                            is_under_dispute: false,
                        },
                    );
                }
            }
            // For dispute, resolve and chargeback, ignore non existing tx IDs and do not modify tx reference.
            Type::Dispute => {
                if let Some(tx) = self
                    .transations_history
                    .get_mut(&transaction.tx)
                    .filter(|tx| !tx.is_under_dispute)
                {
                    self.held += tx.amount;
                    self.available -= tx.amount;
                    tx.is_under_dispute = true;
                }
            }
            Type::Resolve => {
                if let Some(tx) = self
                    .transations_history
                    .get_mut(&transaction.tx)
                    .filter(|tx| tx.is_under_dispute)
                {
                    self.held -= tx.amount;
                    self.available += tx.amount;
                }
            }
            Type::ChargeBack => {
                if let Some(tx) = self
                    .transations_history
                    .get_mut(&transaction.tx)
                    .filter(|tx| tx.is_under_dispute)
                {
                    self.held -= tx.amount;
                    self.total -= tx.amount;
                    self.locked = true;
                }
            }
            _ => {}
        }
    }
}
