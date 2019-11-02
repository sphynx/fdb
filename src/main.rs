use std::io;
use std::io::Write;
use std::process;
use std::str;
use std::str::FromStr;

const PAGE_SIZE: usize = 4096;

// mem::size_of::<Row>() returns 292 because of aligning
const ROW_SIZE: usize = 291;

const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

enum Statement {
    Insert(Row),
    Select,
}

impl FromStr for Statement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("insert") {
            let parts: Vec<&str> = s.split_ascii_whitespace().collect();
            if parts.len() == 4 {
                match parts[1].parse() {
                    Err(_) => Err(format!("Syntax error. Could not parse statement")),
                    Ok(id) => {
                        let row = Row::new(id, parts[2], parts[3]);
                        Ok(Statement::Insert(row))
                    }
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

enum MetaCommand {
    Exit,
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

fn handle_statement(table: &mut Table, stmt: Statement) {
    match stmt {
        Statement::Insert(row) => table.insert(&row),
        Statement::Select => table.select().iter().for_each(Row::print),
    }
}

struct Table {
    num_rows: usize,
    pages: Vec<Box<Page>>,
}

impl Table {
    pub fn new() -> Table {
        Table {
            num_rows: 0,
            pages: vec![],
        }
    }
}

struct Page([u8; PAGE_SIZE]);

const COL_NAME_LENGTH: usize = 32;
const COL_EMAIL_LENGTH: usize = 255;

pub struct Row {
    id: u32,
    name: [u8; COL_NAME_LENGTH],
    email: [u8; COL_EMAIL_LENGTH],
}

impl Row {
    pub fn new(id: u32, name: &str, email: &str) -> Self {
        if name.len() > COL_NAME_LENGTH {
            panic!(
                "Name field should have no more than {} bytes, but has {} bytes",
                COL_NAME_LENGTH,
                name.len()
            );
        }

        if email.len() > COL_EMAIL_LENGTH {
            panic!(
                "Email field should have no more than {} bytes, but has {} bytes",
                COL_EMAIL_LENGTH,
                email.len()
            );
        }

        Row {
            id,
            name: Row::prepare_name(name),
            email: Row::prepare_email(email),
        }
    }

    pub fn print(&self) {
        let name = str::from_utf8(&self.name).unwrap();
        let email = str::from_utf8(&self.email).unwrap();
        println!("({}, {}, {})", self.id, name, email);
    }

    pub fn serialise(&self, where_to: &mut [u8]) {
        let curr = self.id.to_le_bytes();
        let mut start = 0;
        let mut end = start + curr.len();
        where_to[start..end].copy_from_slice(&curr);

        let curr = self.name;
        start = end;
        end = start + curr.len();
        where_to[start..end].copy_from_slice(&curr);

        let curr = self.email;
        start = end;
        end = start + curr.len();
        where_to[start..end].copy_from_slice(&curr);
    }

    pub fn deserialise(where_from: &[u8]) -> Self {
        use std::convert::TryInto;

        let (id_bytes, rest) = where_from.split_at(std::mem::size_of::<u32>());
        let id = u32::from_le_bytes(id_bytes.try_into().unwrap());

        let (name_bytes, rest) = rest.split_at(COL_NAME_LENGTH);
        let name = name_bytes.try_into().unwrap();

        // We can't use try_into() here because the array is greater
        // than 32 elements, so there are not relevant trait
        // implementations for that.
        let (email_bytes, _rest) = rest.split_at(COL_EMAIL_LENGTH);
        let mut email = [0; COL_EMAIL_LENGTH];
        &email.copy_from_slice(email_bytes);

        Row { id, name, email }
    }

    fn prepare_name(name: &str) -> [u8; COL_NAME_LENGTH] {
        let mut array = [0; COL_NAME_LENGTH];
        array[..name.len()].copy_from_slice(name.as_bytes());
        array
    }

    fn prepare_email(email: &str) -> [u8; COL_EMAIL_LENGTH] {
        let mut array = [0; COL_EMAIL_LENGTH];
        array[..email.len()].copy_from_slice(email.as_bytes());
        array
    }
}

impl Table {
    pub fn insert(&mut self, row: &Row) {
        let slot = self.row_slot(self.num_rows);
        row.serialise(slot);
        self.num_rows += 1;
    }

    pub fn select(&mut self) -> Vec<Row> {
        (0..self.num_rows)
            .map(|i| Row::deserialise(self.row_slot(i)))
            .collect()
    }

    fn row_slot(&mut self, row_no: usize) -> &mut [u8] {
        let page_num = row_no / ROWS_PER_PAGE;

        if page_num == self.pages.len() {
            let page = Page([0; PAGE_SIZE]);
            self.pages.push(Box::new(page));
        } else if page_num > self.pages.len() {
            panic!("unexpected row_no: {}", row_no);
        }

        let row_offset = row_no % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        return &mut (self.pages[page_num].0[byte_offset..byte_offset + ROW_SIZE]);
    }
}

fn main() -> io::Result<()> {
    let mut table = Table::new();

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
                Ok(mc) => handle_metacommand(mc),
            }
        } else {
            let statement = Statement::from_str(&command[..]);
            match statement {
                Err(msg) => println!("{}", msg),
                Ok(stmt) => {
                    handle_statement(&mut table, stmt);
                    println!("{}", "Executed.");
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_size_of_row() {
        assert_eq!(std::mem::size_of::<Row>(), 292);
    }

    #[test]
    fn row_serialisation_normal() {
        let row = Row::new(1, "ivan", "veselov@gmail.com");
        check_serialisation(&row);
    }

    #[test]
    fn row_serialisation_empty() {
        let row = Row::new(1, "", "");
        check_serialisation(&row);
    }

    #[test]
    fn row_serialisation_32bytes() {
        let arr = [b'a'; 32];
        let username = std::str::from_utf8(&arr).unwrap();
        let row = Row::new(1, username, "");
        check_serialisation(&row);
    }

    #[test]
    #[should_panic]
    fn row_serialisation_too_large_username() {
        let arr = [b'a'; 33];
        let username = std::str::from_utf8(&arr).unwrap();
        let row = Row::new(1, username, "");
        check_serialisation(&row);
    }

    #[test]
    fn table_insert_select() {
        let mut table = Table::new();
        assert_eq!(table.select().len(), 0);

        let row = Row::new(1, "ivan", "veselov@gmail.com");
        table.insert(&row);
        table.insert(&row);
        assert_eq!(table.select().len(), 2);
    }

    fn check_serialisation(row: &Row) {
        let mut page = [0; PAGE_SIZE];

        row.serialise(&mut page);
        let row2 = Row::deserialise(&page);

        assert_eq!(row.id, row2.id);
        assert_eq!(row.name, row2.name);
        assert_eq!(&row.email[..], &row2.email[..]);
    }
}
