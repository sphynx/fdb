use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::storage::{COL_EMAIL_LENGTH, COL_NAME_LENGTH};

pub enum Statement {
    Insert(u32, String, String),
    Select,
}

pub enum MetaCommand {
    Exit,
}

#[derive(Debug)]
pub enum ParseError {
    UnrecognisedKeyword,
    UnrecognisedMetaCommand,
    SyntaxError,
    StringTooLong,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ParseError::*;
        match self {
            UnrecognisedKeyword => write!(f, "Unrecognised keyword in the statement"),
            UnrecognisedMetaCommand => write!(f, "Unrecognised metacommand"),
            SyntaxError => write!(f, "Syntax error. Could not parse statement"),
            StringTooLong => write!(f, "String is too long"),
        }
    }
}

impl Error for ParseError {}

impl FromStr for Statement {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("insert") {
            let parts: Vec<&str> = s.split_ascii_whitespace().collect();
            if parts.len() == 4 {
                match parts[1].parse() {
                    Err(_) => Err(ParseError::SyntaxError),
                    Ok(id) => {
                        let user = parts[2];
                        let email = parts[3];

                        if user.len() > COL_NAME_LENGTH || email.len() > COL_EMAIL_LENGTH {
                            Err(ParseError::StringTooLong)
                        } else {
                            Ok(Statement::Insert(id, user.to_owned(), email.to_owned()))
                        }
                    }
                }
            } else {
                Err(ParseError::SyntaxError)
            }
        } else if s.trim() == "select" {
            Ok(Statement::Select)
        } else {
            Err(ParseError::UnrecognisedKeyword)
        }
    }
}

impl FromStr for MetaCommand {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "exit" {
            Ok(MetaCommand::Exit)
        } else {
            Err(ParseError::UnrecognisedMetaCommand)
        }
    }
}
