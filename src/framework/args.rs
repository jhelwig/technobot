use nom::{self, ErrorKind, IResult, Needed};
use std::fmt;
use std::str::{self, FromStr};
use std::error::Error as StdError;
use util::arg_parser;

#[derive(Debug)]
pub enum Error<E: StdError> {
    Parse(E),
    Empty,
    ArgParse,
}

impl<E: StdError> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Parse(e)
    }
}

impl<E: StdError> StdError for Error<E> {
    fn description(&self) -> &str {
        match self {
            &Error::ArgParse => "Error parsing argument string",
            &Error::Parse(ref e) => e.description(),
            &Error::Empty => "No remaining unparsed arguments",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::Parse(ref e) => Some(e),
            _ => None,
        }
    }
}

impl<E: StdError> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::ArgParse => write!(f, "Error parsing argument string"),
            &Error::Parse(ref e) => write!(f, "Parse error: {}", e),
            &Error::Empty => write!(f, "No remaining unparsed arguments"),
        }
    }
}

type Result<T, E> = ::std::result::Result<T, Error<E>>;

#[derive(Debug)]
pub struct Args {
    unparsed: String,
}

impl Args {
    pub fn new(arg_str: &str) -> Args {
        Args { unparsed: arg_str.to_string() }
    }

    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError
    {
        if self.unparsed.is_empty() {
            return Err(Error::Empty);
        }

        let to_parse = self.unparsed.clone();
        match arg_parser::single_arg(to_parse.as_bytes()) {
            IResult::Done(rest, arg_str) => {
                let result = arg_str.parse::<T>()?;
                self.unparsed = str::from_utf8(rest).unwrap().to_string();
                Ok(result)
            }
            _ => Err(Error::ArgParse),
        }
    }

    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError
    {
        if self.unparsed.is_empty() {
            return Err(Error::Empty);
        }

        let to_parse = self.unparsed.clone();
        match arg_parser::single_quoted_arg(to_parse.as_bytes()) {
            IResult::Done(rest, arg_str) => {
                let result = arg_str.parse::<T>()?;
                self.unparsed = str::from_utf8(rest).unwrap().to_string();
                Ok(result)
            }
            _ => Err(Error::ArgParse),
        }
    }
}
