#![cfg(test)]

use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;

use kresolver::Book;

#[test]
fn test_sample_from_problem_statement() -> Result<()> {
    let transactions = r"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
";

    let mut book = Book::new();
    let resolved_book = book.try_resolve_all(transactions.to_owned())?;
    let mut buf = vec![];
    resolved_book.write(Cursor::new(&mut buf))?;

    let expected = r"id,available,held,total,locked
1,1.5,0.0000,1.5,false
2,2.0,0.0000,2.0,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}

#[test]
fn test_dispute() -> Result<()> {
    let transactions = r"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
";

    let mut book = Book::new();
    let resolved_book = book.try_resolve_all(transactions.to_owned())?;
    let mut buf = vec![];
    resolved_book.write(Cursor::new(&mut buf))?;

    let expected = r"id,available,held,total,locked
1,0.0000,1.0,1.0,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}

#[test]
fn test_resolve() -> Result<()> {
    let transactions = r"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
deposit, 1, 2, 1.0
resolve, 1, 1,
resolve, 1, 2,
";

    let mut book = Book::new();
    let resolved_book = book.try_resolve_all(transactions.to_owned())?;
    let mut buf = vec![];
    resolved_book.write(Cursor::new(&mut buf))?;

    let expected = r"id,available,held,total,locked
1,2.0,0.0000,2.0,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}

#[test]
fn test_negative() -> Result<()> {
    let transactions = r"type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 2, 1.0
dispute, 1, 1,
";

    let mut book = Book::new();
    let resolved_book = book.try_resolve_all(transactions.to_owned())?;
    let mut buf = vec![];
    resolved_book.write(Cursor::new(&mut buf))?;

    let expected = r"id,available,held,total,locked
1,-1.0,1.0,0.0000,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}

#[test]
fn test_chargeback() -> Result<()> {
    let transactions = r"type, client, tx, amount
deposit, 1, 1, 1.0
dispute, 1, 1,
deposit, 1, 2, 1.0
deposit, 2, 3, 9_001_000.1
chargeback, 1, 1,
";

    let mut book = Book::new();
    let resolved_book = book.try_resolve_all(transactions.to_owned())?;
    let mut buf = vec![];
    resolved_book.write(Cursor::new(&mut buf))?;

    let expected = r"id,available,held,total,locked
1,1.0,0.0000,1.0,true
2,9001000.1,0.0000,9001000.1,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}

#[test]
fn test_thousands() -> Result<()> {
    let mut buf = vec![];
    let path = PathBuf::try_from("tests/350k_transactions.csv")?;
    Book::new()
        .try_resolve_all(path)?
        .write(Cursor::new(&mut buf))?;

    // Evaluated externally with xsv using
    // `xsv search -s client '^9002$' 350k_transactions.csv | xsv select amount | xsv stats | xsv select sum`
    let expected = r"id,available,held,total,locked
1,3531951.4851,0.0000,3531951.4851,false
2,3535651.6485,0.0000,3535651.6485,false
3,3538570.3290,0.0000,3538570.3290,false
4,3526739.5399,0.0000,3526739.5399,false
9001,1764184.4771,0.0000,1764184.4771,false
9002,1765838.5850,0.0000,1765838.5850,false
";

    assert_eq!(&buf, &expected.bytes().collect::<Vec<_>>());

    Ok(())
}
