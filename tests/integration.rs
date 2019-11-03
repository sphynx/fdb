use assert_cmd::prelude::*;
use rexpect::errors::*;
use rexpect::session::spawn_command;
use std::process::Command;

#[test]
fn test_insert_delete() -> Result<()> {
    let timeout = Some(500);
    let mut p = spawn_command(Command::cargo_bin("fdb").unwrap(), timeout)?;
    p.send_line("insert 1 user1 person1@example.com")?;
    p.exp_string("Executed.")?;
    p.send_line("select")?;
    p.exp_string("(1, user1, person1@example.com)")?;
    p.send_line(".exit")?;
    p.exp_eof()?;
    Ok(())
}

#[test]
fn test_many() -> Result<()> {
    let timeout = Some(500);
    let mut p = spawn_command(Command::cargo_bin("fdb").unwrap(), timeout)?;

    for i in 0..1000 {
        p.send_line(&format!("insert {0} user{0} person{0}@example.com", i))?;
    }

    p.send_line("select")?;

    for i in 0..1000 {
        p.exp_string(&format!("({0}, user{0}, person{0}@example.com)", i))?;
    }

    p.send_line(".exit")?;
    p.exp_eof()?;
    Ok(())
}

#[test]
fn test_long() -> Result<()> {
    let timeout = Some(500);
    let mut p = spawn_command(Command::cargo_bin("fdb").unwrap(), timeout)?;

    let long_name = "a".repeat(32);
    let long_email = "b".repeat(255);

    p.send_line(&format!("insert 1 {} {}", long_name, long_email))?;
    p.send_line("select")?;
    p.exp_string(&format!("(1, {}, {})", long_name, long_email))?;
    Ok(())
}
