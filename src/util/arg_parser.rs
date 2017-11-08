use nom::{AsBytes, AsChar, ErrorKind, IResult, InputIter, InputLength, is_space, Needed, Slice,
          space};

use std::ops::{Range, RangeFrom, RangeTo};
use std::str;
use std::string::String;

fn non_quote_special<T>(input: T) -> IResult<T, T>
    where T: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
          T: InputIter + InputLength,
          <T as InputIter>::Item: AsChar
{
    let input_length = input.input_len();
    if input_length == 0 {
        return IResult::Incomplete(Needed::Unknown);
    }

    for (idx, item) in input.iter_indices() {
        let item_char = item.as_char();
        if item_char == '"' || item_char == '\\' {
            if idx == 0 {
                return IResult::Error(error_position!(ErrorKind::IsNot, input));
            } else {
                return IResult::Done(input.slice(idx..), input.slice(0..idx));
            }
        }
    }
    IResult::Done(input.slice(input_length..), input)
}

fn single<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a str> {
    if input.input_len() == 0 {
        return IResult::Incomplete(Needed::Unknown);
    }

    map_res!(input,
             do_parse!(
        arg: take_till!(is_space) >>
        (arg)
    ),
             str::from_utf8)
}

fn single_string<'a>(input: &'a [u8]) -> IResult<&'a [u8], String> {
    match single(input) {
        IResult::Done(r, s) => IResult::Done(r, s.to_string()),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(i) => IResult::Incomplete(i),
    }
}

pub fn single_arg<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a str> {
    if input.input_len() == 0 {
        return IResult::Incomplete(Needed::Unknown);
    }

    do_parse!(input,
        arg: call!(single) >>
        alt!(eof!() | space) >>
        (arg)
    )
}


fn quoted<'a>(input: &'a [u8]) -> IResult<&'a [u8], String> {
    map_res!(input,
             do_parse!(
        arg: delimited!(
            char!('"'),
            escaped_transform!(
                call!(non_quote_special),
                '\\',
                alt!(
                    tag!("\\") => { |_| &b"\\"[..] } |
                    tag!("\"") => { |_| &b"\""[..] }
                )),
            char!('"')) >>
        (arg)
    ),
             String::from_utf8)
}

pub fn single_quoted_arg<'a>(input: &'a [u8]) -> IResult<&'a [u8], String> {
    do_parse!(input,
        arg: alt!(call!(quoted) | call!(single_string)) >>
        alt!(eof!() | space) >>
        (arg)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_arg() {
        assert_eq!(single_arg("".as_bytes()),
                   IResult::Incomplete(Needed::Unknown));
        assert_eq!(single_arg("single".as_bytes()),
                   IResult::Done("".as_bytes(), "single"));
        assert_eq!(single_arg("one two".as_bytes()),
                   IResult::Done("two".as_bytes(), "one"));
        assert_eq!(single_arg("\"one two\"".as_bytes()),
                   IResult::Done("two\"".as_bytes(), "\"one"));
    }

    #[test]
    fn test_single_quoted_arg() {
        assert_eq!(single_quoted_arg("".as_bytes()),
                   IResult::Incomplete(Needed::Size(1)));
        assert_eq!(single_quoted_arg("single".as_bytes()),
                   IResult::Done("".as_bytes(), "single".to_string()));
        assert_eq!(single_quoted_arg("one two".as_bytes()),
                   IResult::Done("two".as_bytes(), "one".to_string()));

        assert_eq!(single_quoted_arg("\"quoted arg\" with more".as_bytes()),
                   IResult::Done("with more".as_bytes(), "quoted arg".to_string()));
        assert_eq!(single_quoted_arg("\"quoted with \\\" escaped\" more".as_bytes()),
                   IResult::Done("more".as_bytes(), "quoted with \" escaped".to_string()));
        assert_eq!(single_quoted_arg("\"quote does not end".as_bytes()),
                   IResult::Incomplete(Needed::Size(20)));
    }

    #[test]
    fn test_quoted() {
        assert_eq!(quoted(&b""[..]), IResult::Incomplete(Needed::Size(1)));

        assert_eq!(quoted(&b"\"quoted thing\" lkj lkj lkj lkj lkj lkj "[..]),
                   IResult::Done(&b" lkj lkj lkj lkj lkj lkj "[..],
                                 "quoted thing".to_string()));

        assert_eq!(quoted(&b"no quotes here"[..]),
                   IResult::Error(error_position!(ErrorKind::Char, "no quotes here".as_bytes())));
    }
}
