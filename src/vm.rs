use std::process;

use crate::parser::MetaCommand;
use crate::parser::Statement;
use crate::storage::Row;
use crate::storage::Table;

pub struct VM {
    table: Table,
}

impl VM {
    pub fn new() -> Self {
        VM {
            table: Table::new(),
        }
    }

    pub fn handle_metacommand(&self, cmd: MetaCommand) {
        match cmd {
            MetaCommand::Exit => process::exit(0),
        }
    }

    pub fn handle_statement(&mut self, stmt: Statement) {
        match stmt {
            Statement::Insert(id, name, email) => {
                let row = Row::new(id, &name, &email);
                self.table.insert(&row);
            }
            Statement::Select => self.table.select().iter().for_each(Row::print),
        }
    }
}
