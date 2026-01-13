# Transactions Simulator

This project implements a simple transaction processing engine that reads a CSV file of client transactions, applies a set of business rules, and outputs the final state of each client account as a CSV written to stdout.

All non-explicit behaviors are handled through clearly documented assumptions to ensure deterministic and well-defined behavior.

---

## Project structure

transactions-simulator

src/

* main.rs
  
  Entry point of the application. It delegates all the processing to the business logic module bubbling up possible errors.

* business_logic/

  * mod.rs
    
    Defines the core data structures (Client, ClientTransaction, Transaction, Type) and contains the logic responsible for parsing input transactions, applying them in order, and writing the final output.
  * transactions_logic.rs
    
    Contains the business rules for applying each transaction type to a client account.
  * trait_impl.rs
    
    Trait implementations used by the business logic, including Serialize / Deserialize helpers and custom formatting for decimal values with four digits of precision.

* shared/

  * errors.rs
    
    Shared error definitions using the thiserror crate.
  * mod.rs

Tests/

* tests/inputs/
  
  CSV input files used for integration-style tests.
* tests/outputs/
  
  Expected CSV outputs corresponding to each input file.

---

## Input format

The input is a CSV file with the following columns:

type, client, tx, amount

* type: string representing the transaction type
* client: u16 client identifier
* tx: u32 transaction identifier (globally unique)
* amount: decimal value with up to four digits of precision (may be empty for some transaction types)

Example:

type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,2.0
withdrawal,1,3,1.5

Whitespace around fields and decimal precision up to four places are accepted.

---

## Output format

The output is written as CSV to stdout and contains the following columns:

client, available, held, total, locked

* available: funds available for withdrawal or trading
* held: funds held due to disputes
* total: available + held
* locked: whether the account is frozen

All monetary values are printed with exactly four decimal places.

Example:

client,available,held,total,locked
1,1.5000,0.0000,1.5000,false

Row ordering is not significant.

---

## Supported transaction types

The following table summarizes the supported transaction types and their effects on a client account:

| Transaction type | Description                                                             | Effect on balances                                   |
| ---------------- | ----------------------------------------------------------------------- | ---------------------------------------------------- |
| deposit          | Adds funds to the client account                                        | available += amount, total += amount                 |
| withdrawal       | Removes funds from the client account if sufficient funds are available | available -= amount, total -= amount                 |
| dispute          | Opens a dispute on a previous transaction                               | held += tx_amount, available -= tx_amount            |
| resolve          | Resolves an open dispute                                                | held -= tx_amount, available += tx_amount            |
| chargeback       | Finalizes a dispute and freezes the account                             | held -= tx_amount, total -= tx_amount, locked = true |

Dispute, resolve, and chargeback transactions refer to a previous transaction via its transaction ID and do not include an amount themselves.

---

## Assumptions

While implementing the solution, the following assumptions were made due to incomplete or ambiguous specifications:

1. Accounts start with zero balance and cannot go negative. Withdrawals with insufficient available funds are ignored.
2. Only deposit and withdrawal transactions are stored in transaction history. Dispute-related operations only reference existing monetary transactions.
3. A dispute can only be applied once to a given transaction. Disputing an already disputed transaction is ignored.
4. If a dispute refers to a transaction ID that does not exist, the dispute is ignored and treated as a partner-side error.
5. Resolve operations are only valid if the referenced transaction exists and is currently under dispute. If the transaction does not exist or is not under dispute, the resolve is ignored and treated as a partner-side error.
6. Chargeback operations are only valid if the referenced transaction exists and is currently under dispute. If the transaction does not exist or is not under dispute, the chargeback is ignored and treated as a partner-side error.
7. After a chargeback occurs, the client account is immediately locked and all subsequent transactions are ignored.
8. If a transaction with a duplicate transaction ID is encountered, it is ignored and the original transaction is preserved.
9. Malformed CSV rows or rows that fail deserialization are ignored.

These assumptions are documented to make the behavior explicit and easy to adjust if required.

---

## Running the program

The program reads a CSV file path as input and writes the resulting CSV to stdout.

Example:

cargo run -- input.csv > output.csv

---

## Running tests

The project includes integration-style tests using real CSV input and expected output files.

To run all tests:

cargo test

Each test feeds an input CSV to the business logic and compares the stdout output against the corresponding expected CSV file.

---

## Notes

The design favors clarity and correctness over premature optimization. Transactions are processed sequentially in file order and client state is maintained in memory using hash maps for efficient lookups.
