use nom::{
    bytes::complete::{tag, take, take_till, take_while},
    character::complete::{anychar, digit1, one_of},
    combinator::{opt, peek},
    Err, IResult, Needed,
};
use std::num::NonZeroUsize;
use unicode_xid::UnicodeXID;

fn parse_count(input: &str) -> IResult<&str, Count> {
    let error = Err(Err::Error(nom::error::Error {
        input: "",
        code: nom::error::ErrorKind::Fail,
    }));
    if input.is_empty() {
        return error;
    }
    if let (new_input, Some(arg)) = opt(parse_identifier)(input)? {
        if let (new_new_input, Some(_)) = opt(tag("$"))(new_input)? {
            Ok((new_new_input, Count::Parameter(Argument::Identifier(arg))))
        } else {
            Err(Err::Error(nom::error::Error {
                input: new_input,
                code: nom::error::ErrorKind::Fail,
            }))
        }
    } else if let (new_input, Some(count)) = opt(digit1)(input)? {
        Ok((new_input, Count::Integer(count.parse::<usize>().unwrap())))
    } else {
        error
    }
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    let (car, cdr) = (
        input
            .chars()
            .nth(0)
            .ok_or(Err::Incomplete(Needed::Size(unsafe {
                NonZeroUsize::new_unchecked(1)
            })))?,
        &input[1..],
    );
    if !UnicodeXID::is_xid_start(car) {
        return Err(Err::Error(nom::error::Error {
            input: "",
            code: nom::error::ErrorKind::Fail,
        }));
    }
    let out: IResult<&str, &str> = take_while(UnicodeXID::is_xid_continue)(cdr);
    let (input, cdr) = out?;
    let out = format!("{}{}", car, cdr);
    return Ok((input, out));
}

fn parse_argument(input: &str) -> IResult<&str, Argument> {
    if let (new_input, Some(arg)) = opt(parse_identifier)(input)? {
        Ok((new_input, Argument::Identifier(arg)))
    } else if let (new_input, Some(arg)) = opt(digit1)(input)? {
        Ok((new_input, Argument::Integer(arg.parse::<usize>().unwrap())))
    } else {
        Err(Err::Error(nom::error::Error {
            input: "",
            code: nom::error::ErrorKind::Fail,
        }))
    }
}

pub fn parse_fmt_spec(input: &str) -> IResult<&str, FormatSlot> {
    // NOTE: macros, maybe?
    let (input, arg) = if let (new_input, Some(ident)) = opt(parse_argument)(input)? {
        (new_input, Some(ident))
    } else {
        (input, None)
    };
    let (input, fmt_spec) = if let (new_input, Some(_)) = opt(tag(":"))(input)? {
        let (new_input, fill) = if let (new_new_input, Some(fill)) = opt(anychar)(new_input)? {
            (new_new_input, Some(fill))
        } else {
            (new_input, None)
        };
        let (new_input, align) =
            if let (new_new_input, Some(align)) = opt(one_of("<^>"))(new_input)? {
                (
                    new_new_input,
                    Some(match align {
                        '<' => Align::Left,
                        '^' => Align::Center,
                        '>' => Align::Right,
                        _ => {
                            unreachable!()
                        }
                    }),
                )
            } else {
                (new_input, None)
            };
        let (new_input, sign) = if let (new_new_input, Some(sign)) = opt(one_of("+-"))(new_input)? {
            (
                new_new_input,
                Some(match sign {
                    '+' => Sign::Positive,
                    '-' => Sign::Negative,
                    _ => {
                        unreachable!()
                    }
                }),
            )
        } else {
            (new_input, None)
        };
        let (new_input, alternate) = if let (new_new_input, Some(_)) = opt(tag("#"))(new_input)? {
            (new_new_input, true)
        } else {
            (new_input, false)
        };
        let (new_input, pad_with_zeros) =
            if let (new_new_input, Some(_)) = opt(tag("0"))(new_input)? {
                (new_new_input, true)
            } else {
                (new_input, false)
            };
        let (new_input, width) = if let (new_new_input, Some(width)) = opt(parse_count)(new_input)?
        {
            (new_new_input, Some(width))
        } else {
            (new_input, None)
        };
        let (new_input, percision) = if let (new_new_input, Some(_)) = opt(tag("."))(new_input)? {
            let new_input = new_new_input;
            if let (new_new_input, Some(percision)) = opt(parse_count)(new_input)? {
                (new_new_input, Some(Percision::Count(percision)))
            } else if let (new_new_input, Some(_)) = opt(tag("*"))(new_input)? {
                (new_new_input, Some(Percision::SpecifiedPercision))
            } else {
                (new_input, None)
            }
        } else {
            (new_input, None)
        };
        let (new_input, kind) =
            if let (new_new_input, Some(kind)) = opt(one_of("?oxXpbeE"))(new_input)? {
                match kind {
                    'x' => {
                        if let (new_new_new_input, Some(_)) = opt(tag("?"))(new_new_input)? {
                            (new_new_new_input, Type::DebugLowerHex)
                        } else {
                            (new_new_input, Type::LowerHex)
                        }
                    }
                    'X' => {
                        if let (new_new_new_input, Some(_)) = opt(tag("?"))(new_new_input)? {
                            (new_new_new_input, Type::DebugUpperHex)
                        } else {
                            (new_new_input, Type::UpperHex)
                        }
                    }
                    _ => (
                        new_new_input,
                        match kind {
                            '?' => Type::Debug,
                            'o' => Type::Octal,
                            'p' => Type::Pointer,
                            'b' => Type::Binary,
                            'e' => Type::LowerExp,
                            'E' => Type::UpperExp,
                            _ => {
                                unreachable!()
                            }
                        },
                    ),
                }
            } else {
                (new_input, Type::None)
            };
        (
            new_input,
            Some(FormatSpec {
                fill,
                align,
                sign,
                alternate,
                pad_with_zeros,
                width,
                percision,
                kind,
            }),
        )
    } else {
        (input, None)
    };
    Ok((input, FormatSlot { arg, fmt_spec }))
}

pub fn parse_fmt_str(input: &str) -> Result<FormatString, &str> {
    let mut strings: Vec<String> = vec![];
    let mut slots: Vec<PossibleFormatSlot> = vec![];
    let mut string: String = "".to_string();
    let mut input: String = input.to_string();
    loop {
        if input.is_empty() {
            break;
        }
        let part1: IResult<&str, &str> = peek(tag("{{"))(&input);
        let part2: IResult<&str, &str> = peek(tag("}}"))(&input);
        if !part1.is_err() || !part2.is_err() {
            let out: IResult<&str, &str> = { take(1usize)(&input) };
            let (input_str, push2str) = out.unwrap();
            strings.push(string);
            slots.push(match push2str {
                "{" => PossibleFormatSlot::LeftBrace,
                "}" => PossibleFormatSlot::RightBrace,
                _ => {
                    unreachable!()
                }
            });
            string = "".to_string();
            let out: IResult<&str, &str> = take(1usize)(input_str);
            input = out.unwrap().1.to_string();
            continue;
        }
        // let out: IResult<&str, &str> = peek(tag("{"))(input);
        // assert!(out.is_err());
        // *cracks knuckles*
        if input.starts_with("{") {
            let out: IResult<&str, &str> = take_till(|chr| chr == '}')(&input);
            let (input_str, format) = out.unwrap();
            let mut input_str = input_str.to_string();
            let mut format = format.to_string();
            if format.is_empty() {
                strings.push(input_str);
                break;
            }
            if !input_str.is_empty() {
                input_str.remove(0);
            }
            format.remove(0);
            if format.is_empty() {
                strings.push(input_str);
                break;
            }
            input = input_str.to_string();
            let next = parse_fmt_spec(&format);
            if next.is_err() {
                return Err("Invalid format string. (slot didn't parse)");
            }
            let (left, slot) = next.unwrap();
            if !left.is_empty() {
                return Err("Invalid format string. (slot had additional data)");
            }
            slots.push(PossibleFormatSlot::FormatSlot(slot));
        } else {
            let cloned_input = input.clone();
            let out: IResult<&str, &str> = take_till(|chr| chr == '{' || chr == '}')(&cloned_input);
            let (input_str, push2str) = out.unwrap();
            input = input_str.to_string();
            strings.push(push2str.to_string());
        }
    }
    Ok(FormatString {
        text: strings,
        maybe_fmt: slots,
    })
}

pub type Fill = char;

#[derive(Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Count {
    Parameter(Argument),
    Integer(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Percision {
    Count(Count),
    SpecifiedPercision,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Debug,
    DebugLowerHex,
    DebugUpperHex,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
    None,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FormatSpec {
    pub fill: Option<Fill>,
    pub align: Option<Align>,
    pub sign: Option<Sign>,
    pub alternate: bool,
    pub pad_with_zeros: bool,
    pub width: Option<Count>,
    pub percision: Option<Percision>,
    pub kind: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Argument {
    Identifier(String),
    Integer(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FormatSlot {
    pub arg: Option<Argument>,
    pub fmt_spec: Option<FormatSpec>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PossibleFormatSlot {
    FormatSlot(FormatSlot),
    LeftBrace,
    RightBrace,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FormatString {
    pub text: Vec<String>,
    pub maybe_fmt: Vec<PossibleFormatSlot>,
}
