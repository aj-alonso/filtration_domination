use std::io;
use std::str::FromStr;

/// Tries to parse the next element from the given iterator.
pub fn parse_next<'a, F: FromStr, I: Iterator<Item = &'a str>>(it: &mut I) -> Result<F, io::Error>
where
    <F as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    parse(it.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Not enough values found when reading a line.",
        )
    })?)
}

/// Tries to parse, and if that fails wraps the error in a [io::Error].
pub(crate) fn parse<F: FromStr>(x: &str) -> Result<F, io::Error>
where
    <F as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    x.parse()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
