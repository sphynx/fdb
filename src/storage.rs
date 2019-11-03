use std::str;

const PAGE_SIZE: usize = 4096;

// mem::size_of::<Row>() returns 292 because of aligning
const ROW_SIZE: usize = 291;

const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

pub struct Table {
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

pub const COL_NAME_LENGTH: usize = 32;
pub const COL_EMAIL_LENGTH: usize = 255;

pub struct Row {
    pub id: u32,
    pub name: [u8; COL_NAME_LENGTH],
    pub email: [u8; COL_EMAIL_LENGTH],
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
        let name = str::from_utf8(&self.name).expect("invalid UTF-8 in name");
        let email = str::from_utf8(&self.email).expect("invalid UTF-8 in email");
        println!(
            "({}, {}, {})",
            self.id,
            name.trim_end_matches(char::from(0)),
            email.trim_end_matches(char::from(0))
        );
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
        let username = "a".repeat(32);
        let row = Row::new(1, &username, "");
        check_serialisation(&row);
    }

    #[test]
    #[should_panic]
    fn row_serialisation_too_large_username() {
        let username = "a".repeat(33);
        let row = Row::new(1, &username, "");
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
        let mut page = [0; 4096];

        row.serialise(&mut page);
        let row2 = Row::deserialise(&page);

        assert_eq!(row.id, row2.id);
        assert_eq!(row.name, row2.name);
        assert_eq!(&row.email[..], &row2.email[..]);
    }
}
