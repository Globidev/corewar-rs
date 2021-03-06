mod combinator;

use super::{
    lexer::{LexerError, NumberBase, Term, Token, TokenResult, Tokenizer},
    types::*,
};
use combinator::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedLine {
    ChampionName(String),
    ChampionComment(String),
    Code(Vec<u8>),
    Op(Op),
    Label(String),
    LabelAndOp(String, Op),
    Empty,
}

pub fn parse_line(input: &str) -> Result<ParsedLine, ParseError> {
    let mut tokens = TokenStream::new(input);

    let first_tok = match tokens.peek() {
        None => return Ok(ParsedLine::Empty),
        Some(tok_result) => tok_result.clone()?,
    };

    let parse_result = match first_tok.term {
        Term::ChampionNameCmd => champion_name(&mut tokens).map(ParsedLine::ChampionName),
        Term::ChampionCommentCmd => champion_comment(&mut tokens).map(ParsedLine::ChampionComment),
        Term::CodeCmd => code(&mut tokens).map(ParsedLine::Code),
        Term::LabelDef => {
            let label = label(&mut tokens)?;

            let parsed = match tokens.peek() {
                None
                | Some(Ok(Token {
                    term: Term::Comment,
                    ..
                })) => ParsedLine::Label(label),
                _ => ParsedLine::LabelAndOp(label, op(&mut tokens)?),
            };

            Ok(parsed)
        }
        Term::Ident => op(&mut tokens).map(ParsedLine::Op),
        Term::Comment => return Ok(ParsedLine::Empty),
        _ => return Err(ParseError::Unexpected(first_tok)),
    }?;

    // Remaining input besides comments => Error
    match tokens.peek() {
        None
        | Some(Ok(Token {
            term: Term::Comment,
            ..
        })) => Ok(parse_result),
        Some(Ok(token)) => Err(ParseError::RemainingInput(token.clone())),
        Some(Err(lex_error)) => Err(ParseError::LexerError(lex_error.clone())),
    }
}

type ParseResult<T> = Result<T, ParseError>;

fn champion_name(input: &mut TokenStream<'_>) -> ParseResult<String> {
    input.next(Term::ChampionNameCmd)?;
    input.next(Term::QuotedString).map(String::from)
}

fn champion_comment(input: &mut TokenStream<'_>) -> ParseResult<String> {
    input.next(Term::ChampionCommentCmd)?;
    input.next(Term::QuotedString).map(String::from)
}

fn code(input: &mut TokenStream<'_>) -> ParseResult<Vec<u8>> {
    input.next(Term::CodeCmd)?;
    let numbers = number.many().parse(input)?;
    // TODO: Better enforce numeric limit invariants
    let as_bytes = numbers.into_iter().map(|x| x as u8).collect();
    Ok(as_bytes)
}

fn label(input: &mut TokenStream<'_>) -> ParseResult<String> {
    input
        .next(Term::LabelDef)
        .map(|label_str| String::from(&label_str[..label_str.len() - 1]))
}

fn label_param(input: &mut TokenStream<'_>) -> ParseResult<String> {
    input
        .next(Term::LabelUse)
        .map(|label_str| String::from(&label_str[1..]))
}

fn number(input: &mut TokenStream<'_>) -> ParseResult<i64> {
    let token_result = input.tokens.next().ok_or_else(|| {
        expected_either((
            ParseError::ExpectedButGotEof(Term::Number {
                base: NumberBase::Decimal,
            }),
            ParseError::ExpectedButGotEof(Term::Number {
                base: NumberBase::Hexadecimal,
            }),
        ))
    })?;

    let tok = token_result?;

    match tok.clone() {
        Token {
            term: Term::Number { base },
            range,
        } => {
            let mut number_as_str = &input.input[range];
            let negative = number_as_str.starts_with('-');
            if negative {
                number_as_str = &number_as_str[1..];
            }
            if number_as_str.starts_with("0d") || number_as_str.starts_with("0x") {
                number_as_str = &number_as_str[2..];
            }

            i64::from_str_radix(number_as_str, base.radix())
                .map_err(|e| ParseError::ParseIntError(e, tok))
                .map(|x| if negative { -x } else { x })
        }
        _ => Err(expected_either((
            ParseError::ExpectedButGot(
                Term::Number {
                    base: NumberBase::Decimal,
                },
                tok.clone(),
            ),
            ParseError::ExpectedButGot(
                Term::Number {
                    base: NumberBase::Hexadecimal,
                },
                tok,
            ),
        ))),
    }
}

fn register(input: &mut TokenStream<'_>) -> ParseResult<Register> {
    let (tok, reg_str) = input.next_with_token(Term::Ident)?;
    let mut chars = reg_str.chars();

    let first_char = {
        let tok = tok.clone();
        chars.next().ok_or(ParseError::MissingRegisterPrefix(tok))?
    };
    let reg_num_result = chars.as_str().parse();

    match (first_char, reg_num_result) {
        ('r', Ok(x)) if (1..=16).contains(&x) => Ok(Register(x as u8)),
        ('r', Ok(x)) => Err(ParseError::InvalidRegisterCount(x, tok)),
        ('r', Err(e)) => Err(ParseError::RegisterParseIntError(e, tok)),
        (c, _) => Err(ParseError::InvalidRegisterPrefix(c, tok)),
    }
}

fn direct(input: &mut TokenStream<'_>) -> ParseResult<Direct> {
    input.next(Term::DirectChar)?;
    label_param
        .map(Direct::Label)
        .or(number.map(Direct::Numeric))
        .map_err(expected_either)
        .parse(input)
}

fn indirect(input: &mut TokenStream<'_>) -> ParseResult<Indirect> {
    label_param
        .map(Indirect::Label)
        .or(number.map(Indirect::Numeric))
        .map_err(expected_either)
        .parse(input)
}

fn reg_dir(input: &mut TokenStream<'_>) -> ParseResult<RegDir> {
    register
        .map(RegDir::Reg)
        .or(direct.map(RegDir::Dir))
        .map_err(expected_either)
        .parse(input)
}

fn reg_ind(input: &mut TokenStream<'_>) -> ParseResult<RegInd> {
    register
        .map(RegInd::Reg)
        .or(indirect.map(RegInd::Ind))
        .map_err(expected_either)
        .parse(input)
}

fn dir_ind(input: &mut TokenStream<'_>) -> ParseResult<DirInd> {
    direct
        .map(DirInd::Dir)
        .or(indirect.map(DirInd::Ind))
        .map_err(expected_either)
        .parse(input)
}

fn any_param(input: &mut TokenStream<'_>) -> ParseResult<AnyParam> {
    register
        .map(AnyParam::Reg)
        .or(direct.map(AnyParam::Dir))
        .or(indirect.map(AnyParam::Ind))
        .map_err(|((e1, e2), e3)| ParseError::ExpectedOneOf(vec![e1, e2, e3]))
        .parse(input)
}

fn op(input: &mut TokenStream<'_>) -> ParseResult<Op> {
    macro_rules! parse_op {
        ( $op:expr, $p:expr $(,$ps:expr )* ) => {
            Ok($op(
                $p(input)?
                $(, { input.next(Term::ParamSeparator)?; $ps(input)? })*
            ))
        };
    }

    let (tok, mnemonic) = input.next_with_token(Term::Ident)?;

    match mnemonic {
        "live" => parse_op!(Op::Live, direct),
        "ld" => parse_op!(Op::Ld, dir_ind, register),
        "st" => parse_op!(Op::St, register, reg_ind),
        "add" => parse_op!(Op::Add, register, register, register),
        "sub" => parse_op!(Op::Sub, register, register, register),
        "and" => parse_op!(Op::And, any_param, any_param, register),
        "or" => parse_op!(Op::Or, any_param, any_param, register),
        "xor" => parse_op!(Op::Xor, any_param, any_param, register),
        "zjmp" => parse_op!(Op::Zjmp, direct),
        "ldi" => parse_op!(Op::Ldi, any_param, reg_dir, register),
        "sti" => parse_op!(Op::Sti, register, any_param, reg_dir),
        "fork" => parse_op!(Op::Fork, direct),
        "lld" => parse_op!(Op::Lld, dir_ind, register),
        "lldi" => parse_op!(Op::Lldi, any_param, reg_dir, register),
        "lfork" => parse_op!(Op::Lfork, direct),
        "aff" => parse_op!(Op::Aff, register),

        _ => Err(ParseError::InvalidOpMnemonic(String::from(mnemonic), tok)),
    }
}

#[derive(Clone)]
struct TokenStream<'a> {
    tokens: ::std::iter::Peekable<Tokenizer<'a>>,
    input: &'a str,
}

impl TokenStream<'_> {
    fn new(input: &str) -> TokenStream<'_> {
        TokenStream {
            tokens: Tokenizer::new(input).peekable(),
            input,
        }
    }

    fn peek(&mut self) -> Option<&TokenResult> {
        self.tokens.peek()
    }

    fn next(&mut self, term: Term) -> ParseResult<&str> {
        self.next_with_token(term).map(|(_, s)| s)
    }

    fn next_with_token(&mut self, term: Term) -> ParseResult<(Token, &str)> {
        let token_result = self
            .tokens
            .next()
            .ok_or(ParseError::ExpectedButGotEof(term))?;

        let token = token_result?;

        if token.term == term {
            Ok((token.clone(), &self.input[token.range]))
        } else {
            Err(ParseError::ExpectedButGot(term, token))
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseError {
    RemainingInput(Token),
    LexerError(#[from] LexerError),
    Unexpected(Token),
    ExpectedButGot(Term, Token),
    ExpectedButGotEof(Term),
    ExpectedOneOf(Vec<ParseError>),
    InvalidRegisterCount(i64, Token),
    InvalidRegisterPrefix(char, Token),
    MissingRegisterPrefix(Token),
    ParseIntError(std::num::ParseIntError, Token),
    RegisterParseIntError(std::num::ParseIntError, Token),
    InvalidOpMnemonic(String, Token),
}

fn expected_either((e1, e2): (ParseError, ParseError)) -> ParseError {
    ParseError::ExpectedOneOf(vec![e1, e2])
}

use std::fmt;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::collections::HashSet;
        use ParseError::*;

        fn flatten_errors(errs: &[ParseError]) -> Vec<&ParseError> {
            errs.iter()
                .flat_map(|err| match err {
                    ExpectedOneOf(sub_errors) => flatten_errors(sub_errors),
                    _ => vec![err],
                })
                .collect()
        }

        match self {
            RemainingInput(token) => write!(f, "Extra token remaining: '{}'", token.term),
            LexerError(err) => write!(f, "{}", err.kind),
            Unexpected(token) => write!(f, "Unexpected initial token: '{}'", token.term),
            ExpectedButGot(term, token) => {
                write!(f, "Expected '{}' but got '{}'", term, token.term)
            }
            ExpectedButGotEof(term) => write!(f, "Expected '{}' before the end of the line", term),
            ExpectedOneOf(errors) => {
                let unique_error_strings = flatten_errors(errors)
                    .into_iter()
                    .map(|err| format!("{}", err))
                    .collect::<HashSet<_>>();

                match unique_error_strings.len() {
                    0 => unreachable!("ExpectedOneOf has been given an empty sequence"),
                    1 => write!(f, "{}", unique_error_strings.into_iter().next().unwrap()),
                    _ => {
                        let line_separated_erros = unique_error_strings
                            .into_iter()
                            .map(|s| format!("\n- {}", s))
                            .collect::<String>();
                        write!(f, "Either:{}", line_separated_erros)
                    }
                }
            }
            InvalidRegisterCount(n, _) => write!(
                f,
                "'{}' is not a valid register number. It must be between 1 and 16",
                n
            ),
            InvalidRegisterPrefix(prefix, _) => {
                write!(f, "Register prefix should be 'r' and not '{}'", prefix)
            }
            MissingRegisterPrefix(_) => write!(f, "Register prefix 'r' is missing"),
            ParseIntError(err, _) => write!(f, "Invalid number: {}", err),
            RegisterParseIntError(err, _) => write!(f, "Invalid register number: {}", err),
            InvalidOpMnemonic(mnemonic, _) => write!(f, "'{}' is not a valid operation", mnemonic),
        }
    }
}

pub fn error_range(err: &ParseError) -> (usize, Option<usize>) {
    use ParseError::*;

    match err {
        LexerError(err) => (err.at.start, Some(err.at.end)),
        ExpectedButGotEof(_) => (0, None),
        ExpectedOneOf(errors) => {
            let ranges = errors.iter().map(error_range).collect::<Vec<_>>();

            let min_start = ranges
                .iter()
                .map(|(start, _)| *start)
                .min()
                .expect("ExpectedOneOf has been given an empty sequence");

            let max_end = ranges.iter().flat_map(|(_, opt_end)| *opt_end).max();

            (min_start, max_end)
        }
        RemainingInput(token)
        | Unexpected(token)
        | ExpectedButGot(_, token)
        | InvalidRegisterCount(_, token)
        | InvalidRegisterPrefix(_, token)
        | MissingRegisterPrefix(token)
        | ParseIntError(_, token)
        | RegisterParseIntError(_, token)
        | InvalidOpMnemonic(_, token) => (token.range.start, Some(token.range.end)),
    }
}
