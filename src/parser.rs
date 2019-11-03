use std::str::FromStr;

pub enum Statement {
    Insert(u32, String, String),
    Select,
}

pub enum MetaCommand {
    Exit,
}

impl FromStr for Statement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("insert") {
            let parts: Vec<&str> = s.split_ascii_whitespace().collect();
            if parts.len() == 4 {
                match parts[1].parse() {
                    Err(_) => Err(format!("Syntax error. Could not parse statement")),
                    Ok(id) => Ok(Statement::Insert(
                        id,
                        parts[2].to_owned(),
                        parts[3].to_owned(),
                    )),
                }
            } else {
                Err(format!("Syntax error. Could not parse statement"))
            }
        } else if s.trim() == "select" {
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
