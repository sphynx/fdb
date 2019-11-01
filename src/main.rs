use std::io;
use std::io::Write;
use std::process;
use std::str::FromStr;

enum Statement {
    Insert,
    Select,
}

enum MetaCommand {
    Exit,
}

impl FromStr for Statement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("insert") {
            Ok(Statement::Insert)
        } else if s.starts_with("select") {
            Ok(Statement::Select)
        } else {
            Err(format!("Unrecognized keyword at start of: '{}'.", s))
        }
    }
}

impl FromStr for MetaCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "exit" {
            Ok(MetaCommand::Exit)
        } else {
            Err(format!("Unrecognized command: '{}'.", s))
        }
    }
}

fn handle_metacommand(cmd: MetaCommand) {
    match cmd {
        MetaCommand::Exit => process::exit(0),
    }
}

fn handle_statement(stmt: Statement) {
    match stmt {
        Statement::Insert => println!("This is where we would do an insert."),
        Statement::Select => println!("This is where we would do a select."),
    }
}

fn main() -> io::Result<()> {
    loop {
        print!("db > ");
        io::stdout().flush()?;

        let mut command = String::new();
        io::stdin().read_line(&mut command)?;
        command.truncate(command.trim_end().len());

        if command.starts_with(".") {
            let meta_cmd = MetaCommand::from_str(&command[1..]);
            match meta_cmd {
                Err(msg) => println!("{}", msg),
                Ok(mc) => handle_metacommand(mc),
            }
        } else {
            let statement = Statement::from_str(&command[..]);
            match statement {
                Err(msg) => println!("{}", msg),
                Ok(stmt) => {
                    handle_statement(stmt);
                    println!("{}", "Executed.");
                }
            }
        }
    }
}
