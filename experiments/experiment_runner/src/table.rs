use anyhow::Result;
use std::io::Write;
use std::time::Duration;

const MISSING_VALUE_STRING: &str = "-";

pub trait Row {
    fn headers() -> Vec<&'static str>;

    fn fields(&self) -> Vec<Option<String>>;
}

#[derive(Debug, Default, Clone)]
pub struct Table<R> {
    rows: Vec<R>,
}

impl<R: Row> Table<R> {
    pub fn new(rows: Vec<R>) -> Self {
        Self { rows }
    }

    pub fn display_as_csv<W: Write>(&self, w: &mut W) -> Result<()> {
        for (idx, header) in R::headers().iter().enumerate() {
            if idx != 0 {
                write!(w, ",")?;
            }
            write!(w, "{}", header)?;
        }
        writeln!(w)?;
        for row in &self.rows {
            for (idx, field) in row.fields().iter().enumerate() {
                let field: &str = field.as_ref().map(|s| s.as_str()).unwrap_or("");
                if idx != 0 {
                    write!(w, ",")?;
                }
                write!(w, "{}", field)?;
            }
            writeln!(w)?;
        }

        Ok(())
    }
}

pub fn display<T: std::fmt::Display>(a: T) -> String {
    format!("{}", a)
}

pub fn display_option_as<T, F>(a: Option<T>, f: F) -> String
where
    F: FnOnce(T) -> String,
{
    if let Some(a) = a {
        f(a)
    } else {
        String::from(MISSING_VALUE_STRING)
    }
}

pub fn display_option<T: std::fmt::Display>(a: Option<T>) -> String {
    display_option_as(a, display)
}

pub fn display_duration(d: &Duration) -> String {
    format!("{:.2}", d.as_secs_f64())
}
