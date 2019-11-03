use std::io;
use std::io::Write;
use std::str::FromStr;

use fdb::parser::{MetaCommand, Statement};
use fdb::vm::VM;

fn main() -> io::Result<()> {
    let mut vm = VM::new();

    loop {
        print!("db > ");
        io::stdout().flush()?;

        let mut command = String::new();
        io::stdin().read_line(&mut command)?;

        if command.len() == 0 {
            println!("Good bye!");
            return Ok(());
        }

        command.truncate(command.trim_end().len());
        if command.starts_with(".") {
            let meta_cmd = MetaCommand::from_str(&command[1..]);
            match meta_cmd {
                Err(msg) => println!("{}", msg),
                Ok(mc) => vm.handle_metacommand(mc),
            }
        } else {
            let statement = Statement::from_str(&command[..]);
            match statement {
                Err(msg) => println!("{}", msg),
                Ok(stmt) => {
                    vm.handle_statement(stmt);
                    println!("{}", "Executed.");
                }
            }
        }
    }
}
