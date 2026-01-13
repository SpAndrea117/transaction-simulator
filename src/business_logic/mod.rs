use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    business_logic::trait_impl::{four_decimals, from_str},
    shared::errors::Error,
};

#[derive(Debug, Deserialize)]
struct ClientTransaction {
    /// Client ID, UUID
    #[serde(rename = "client")]
    id: u16,
    /// Type of transaction
    #[serde(rename = "type", deserialize_with = "from_str")]
    transaction_type: Type,
    /// Transaction ID
    tx: u32,
    /// Transaction amount. Present only for Deposit and Withdrawal
    amount: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
struct Client {
    /// Client ID, UUID
    #[serde(rename = "client")]
    id: u16,
    /// Available founds = total - held
    #[serde(serialize_with = "four_decimals")]
    available: f64,
    /// Held founds = total - available
    #[serde(serialize_with = "four_decimals")]
    held: f64,
    /// Total founds = available + held
    #[serde(serialize_with = "four_decimals")]
    total: f64,
    /// Identify if client account is locked
    locked: bool,
    #[serde(skip)]
    /// History of transactions of client identified by ID
    transations_history: HashMap<u32, Transaction>,
}

#[derive(Debug)]
struct Transaction {
    /// The found amount linked to this transaction
    amount: f64,
    /// Identify if transaction is under dispute
    is_under_dispute: bool,
}

#[derive(Debug, Deserialize)]
enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    ChargeBack,
}

mod trait_impl;
mod transactions_logic;

/**
 * Having CSV input line, here data are processed as follow:
 * | type [String] | client [UUID - u16] | tx [u32] | amount [f64] |
 *
 * type can be one the following operations:
 *     - deposit:
 *         * available += amount
 *         * total += amount
 *     - withdrawal:
 *         * available -= amount
 *         * total -= amount
 *     - dispute: [Request to revert a transaction. It refers to a specific transaction ID and does not have an amount]
 *         * held += tx_hisotry[tx_id]
 *         * available -= tx_hisotry[tx_id]
 *         * set tx_id attribute is_dispute to true
 *     - resolve: [Request to resolve a dispute. It refers to a specific transaction ID and does not have an amount]
 *         * held -= tx_hisotry[tx_id]
 *         * available += tx_hisotry[tx_id]
 *         * Previous operations should be taken into account iff tx_id exists in history_tx and if tx_id is under dispute
 *     - chargeback: [Final state of a dispute. It refers to a specific transaction ID and does not have an amount]
 *         * held -= tx_hisotry[tx_id]
 *         * total -= tx_hisotry[tx_id]
 *         * Previous operations should be taken into account iff tx_id exists in history_tx and if tx_id is under dispute
 *         * This operation immediately freeze the client acount.
 *
 *  Assumptions to very:
 *  While implementing the solution I identified a few edge cases that are not fully specified. I made conservative assumptions and I’d like to confirm they align with your expectations.
 *  1. I assume accounts start with zero balance and cannot go negative. Withdrawals with insufficient available funds are ignored.
 *  2. I assume disputes can only target previous monetary transactions (deposit/withdrawal), and that disputes themselves cannot be disputed.
 *  3. After a chargeback I treat the account as frozen and ignore all subsequent transactions, considering it a terminal state.
 *  4. If a transaction with an already-seen transaction ID is received, I ignore it and keep the original transaction unchanged.
 *  5. Malformed input data are simply ignored
 *
 *
 *
 *  OUTPUT file will contain
 *  | client [UUID - u16] | available [f64 {.4}] | held [f64 {.4}] | total [f64 {.4}] | locked [bool]|
 *
 */
pub(crate) fn apply_transaction<W>(input_file: PathBuf, writer: W) -> Result<(), Error>
where
    W: Write,
{
    let file = File::open(input_file).map_err(Error::Io)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut client_out = HashMap::<u16, Client>::new();

    for result in reader.deserialize::<ClientTransaction>() {
        let client_transaction = match result {
            Ok(client_tx) => client_tx,
            Err(_) => continue, // ignore malformed input lines
        };

        client_out
            .entry(client_transaction.id)
            .and_modify(|client| client.apply_transaction(&client_transaction))
            .or_insert(Client::from(client_transaction));
    }

    let mut writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    client_out
        .values()
        .into_iter()
        .try_for_each(|client| -> Result<(), Error> {
            writer.serialize(client).map_err(Error::Csv)
        })?;

    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read, path::PathBuf};

    use crate::business_logic::apply_transaction;

    fn check_result(input_file: PathBuf, output_file: PathBuf) {
        let mut buf = Vec::new();
        apply_transaction(input_file, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let mut expected_out = "".to_owned();
        File::open(output_file)
            .unwrap()
            .read_to_string(&mut expected_out)
            .unwrap();

        assert_eq!(output, expected_out)
    }

    #[test]
    fn test_basic() {
        check_result(
            PathBuf::from("./tests/inputs/input_01_basic.csv"),
            PathBuf::from("./tests/outputs/expected_output_01_basic.csv"),
        );
    }

    #[test]
    fn test_insufficient_funds() {
        check_result(
            PathBuf::from("./tests/inputs/input_02_insufficient_funds.csv"),
            PathBuf::from("./tests/outputs/expected_output_02_insufficient_funds.csv"),
        );
    }

    #[test]
    fn test_dispute_resolve() {
        check_result(
            PathBuf::from("./tests/inputs/input_03_dispute_resolve.csv"),
            PathBuf::from("./tests/outputs/expected_output_03_dispute_resolve.csv"),
        );
    }

    #[test]
    fn test_chargeback() {
        check_result(
            PathBuf::from("./tests/inputs/input_04_chargeback.csv"),
            PathBuf::from("./tests/outputs/expected_output_04_chargeback.csv"),
        );
    }

    #[test]
    fn test_invalid_dispute() {
        check_result(
            PathBuf::from("./tests/inputs/input_05_invalid_dispute.csv"),
            PathBuf::from("./tests/outputs/expected_output_05_invalid_dispute.csv"),
        );
    }

    #[test]
    fn test_duplicate_tx() {
        check_result(
            PathBuf::from("./tests/inputs/input_06_duplicate_tx.csv"),
            PathBuf::from("./tests/outputs/expected_output_06_duplicate_tx.csv"),
        );
    }

    #[test]
    fn test_whitespace_precision() {
        check_result(
            PathBuf::from("./tests/inputs/input_07_whitespace_precision.csv"),
            PathBuf::from("./tests/outputs/expected_output_07_whitespace_precision.csv"),
        );
    }

    #[test]
    fn test_dispute_on_dispute() {
        check_result(
            PathBuf::from("./tests/inputs/input_08_dispute_on_dispute.csv"),
            PathBuf::from("./tests/outputs/expected_output_08_dispute_on_dispute.csv"),
        );
    }

    #[test]
    fn test_fuzz_malformed() {
        check_result(
            PathBuf::from("./tests/inputs/fuzz_malformed.csv"),
            PathBuf::from("./tests/outputs/expected_output_fuzz_malformed.csv"),
        );
    }
}
