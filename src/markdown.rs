use std::fmt::{self, Display, Write};
use std::iter::Extend;

pub trait Element: Display {}

pub struct Document {
    buff: String,
}

impl Document {
    pub fn new() -> Document {
        Document { buff: String::new() }
    }

    pub fn add<E>(&mut self, elem: E)
    where
        E: Element,
    {
        let _ = write!(self.buff, "{}", elem);
    }

    pub fn as_str(&self) -> &str {
        &self.buff[..]
    }
}

// Elements

/// Paragraph
pub struct Paragraph(String);

impl Paragraph {
    pub fn new<D>(content: D) -> Paragraph
    where
        D: Display,
    {
        Paragraph(content.to_string())
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n\n", self.0)
    }
}

impl Element for Paragraph {}

/// H1 header
pub struct H1(String);

impl H1 {
    pub fn new<D>(content: D) -> H1
    where
        D: Display,
    {
        H1(content.to_string())
    }
}

impl Display for H1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "# {}\n", self.0)
    }
}

impl Element for H1 {}

/// H2 header
pub struct H2(String);

impl H2 {
    pub fn new<D>(content: D) -> H2
    where
        D: Display,
    {
        H2(content.to_string())
    }
}

impl Display for H2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "## {}\n", self.0)
    }
}

impl Element for H2 {}

/// H3 header
pub struct H3(String);

impl H3 {
    pub fn new<D>(content: D) -> H3
    where
        D: Display,
    {
        H3(content.to_string())
    }
}

impl Display for H3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "### {}\n", self.0)
    }
}

impl Element for H3 {}

/// List
pub struct List(Vec<String>);

impl List {
    pub fn new() -> List {
        List(Vec::new())
    }

    pub fn add<D>(&mut self, item: D)
    where
        D: Display,
    {
        self.0.push(item.to_string())
    }
}

impl<D> Extend<D> for List
where
    D: Display,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = D>,
    {
        for item in iter.into_iter() {
            self.add(item);
        }
    }
}

impl<D, I> From<I> for List
where
    D: Display,
    I: IntoIterator<Item = D>,
{
    fn from(iter: I) -> List {
        let mut list = List::new();
        list.extend(iter);
        list
    }
}

impl Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in self.0.iter() {
            write!(f, "* {}\n", item)?;
        }
        write!(f, "\n")
    }
}

impl Element for List {}

/// Numbered List
pub struct NumberedList(Vec<String>);

impl NumberedList {
    pub fn new() -> NumberedList {
        NumberedList(Vec::new())
    }

    pub fn add<D>(&mut self, item: D)
    where
        D: Display,
    {
        self.0.push(item.to_string())
    }
}

impl<D> Extend<D> for NumberedList
where
    D: Display,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = D>,
    {
        for item in iter.into_iter() {
            self.add(item);
        }
    }
}

impl Display for NumberedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{}. {}\n", index + 1, item)?;
        }
        write!(f, "\n")
    }
}

impl Element for NumberedList {}

/// Table
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new<D, I>(headers: I) -> Table
    where
        D: Display,
        I: IntoIterator<Item = D>,
    {
        Table {
            headers: headers.into_iter().map(|h| h.to_string()).collect(),
            rows: vec![],
        }
    }

    pub fn add<D, I>(&mut self, row: I)
    where
        D: Display,
        I: IntoIterator<Item = D>,
    {
        let row: Vec<_> = row.into_iter().map(|i| i.to_string()).collect();
        if row.len() != self.headers.len() {
            panic!("number of rows is not the same as the number of headers");
        }
        self.rows.push(row);
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, header) in self.headers.iter().enumerate() {
            if index > 0 {
                write!(f, "|{}", header)?;
            } else {
                write!(f, "{}", header)?;
            }
        }
        write!(f, "\n")?;

        for (index, _) in self.headers.iter().enumerate() {
            if index > 0 {
                write!(f, "|:---:")?;
            } else {
                write!(f, ":---:")?;
            }
        }
        write!(f, "\n")?;

        for row in self.rows.iter() {
            for (index, item) in row.iter().enumerate() {
                if index > 0 {
                    write!(f, "|{}", item)?;
                } else {
                    write!(f, "{}", item)?;
                }
            }
            write!(f, "\n")?;
        }

        write!(f, "\n")
    }
}

impl Element for Table {}

/// Code
pub struct Code(String);

impl Code {
    pub fn new<D>(content: D) -> Code
    where
        D: Display,
    {
        Code(content.to_string())
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for line in self.0.lines() {
            write!(f, "    {}\n", line)?;
        }
        write!(f, "\n")
    }
}

impl Element for Code {}

pub struct HR;

impl Display for HR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "---")
    }
}

impl Element for HR {}

#[test]
fn test_h1() {
    let mut doc = Document::new();
    doc.add(H1::new("hello"));
    assert_eq!(doc.as_str(), "# hello\n");
}

#[test]
fn test_h2() {
    let mut doc = Document::new();
    doc.add(H2::new("hello"));
    assert_eq!(doc.as_str(), "## hello\n");
}

#[test]
fn test_h3() {
    let mut doc = Document::new();
    doc.add(H3::new("hello"));
    assert_eq!(doc.as_str(), "### hello\n");
}

#[test]
fn test_list() {
    let mut doc = Document::new();
    let mut list = List::new();
    list.extend(&["hello", "world"]);
    doc.add(list);
    assert_eq!(doc.as_str(), "* hello\n* world\n\n");
}

#[test]
fn test_numbered_list() {
    let mut doc = Document::new();
    let mut list = NumberedList::new();
    list.extend(&["hello", "world"]);
    doc.add(list);
    assert_eq!(doc.as_str(), "1. hello\n2. world\n\n");
}

#[test]
fn test_table_format() {
    let mut doc = Document::new();
    let mut table = Table::new(&["and", "T", "F"]);
    table.add(&["T", "T", "F"]);
    table.add(&["F", "F", "F"]);
    doc.add(table);
    assert_eq!(doc.as_str(), "and|T|F\n:---:|:---:|:---:\nT|T|F\nF|F|F\n\n");
}

#[test]
fn test_code() {
    let mut doc = Document::new();
    doc.add(Code::new("let x = 3;\nlet y = x**2;\n"));
    assert_eq!(doc.as_str(), "\n    let x = 3;\n    let y = x**2;\n\n");
}
