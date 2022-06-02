
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{Token, TMeta};

pub fn tokenize( input : &str ) -> Result<Vec<Token>, String> {
    match internal_tokenize(input) {
        Ok(ts) => {
            Ok(ts.into_iter()
                 .filter(|t| matches!(t, I::T(_)))
                 .map(|t| match t {
                     I::T(x) => x,
                     _ => panic!("Encountered Junk after filter")
                 })
                 .collect())
         },
        Err(MatchError::ErrorEndOfFile | MatchError::FatalEndOfFile) =>
            Err("Encountered unexpected end of file while tokenizing.".into()),
        Err(MatchError::Error(i) | MatchError::Fatal(i)) => {
            let error_reporter::ErrorReport { line, column, display } = error_reporter::report(input, i, i);
            let ret = format!("Encountered tokenization error at line {line} and column {column}:\n\n{display}");
            Err(ret)
        },
    }
}

#[derive(Debug)]
enum I {
    T(Token),
    Junk,
}

fn internal_tokenize( input : &str ) -> Result<Vec<I>, MatchError> {
    let mut x = input.char_indices().enumerate();

    alt!( token: (usize, char) => I = junk 
                                    | lower_symbol 
                                    | upper_symbol
                                    | string
                                    | number
                                    | punctuation
                                    );

    let mut ret = vec![];
    loop {
        match token(&mut x) {
            Ok(t) => ret.push(t),
            Err(MatchError::ErrorEndOfFile) => break,
            Err(e) => return Err(e),
        }
    }

    Ok(ret)
}

group!(junk: (usize, char) => I = |input| {
    pred!(ws: (usize, char) => () = |_c| _c.1.is_whitespace() => { () });
    seq!(whitespace: (usize, char) => I = ws, * ws, { I::Junk });

    pred!(end_line: (usize, char) => () = |_c| _c.1 == '\n' || _c.1 == '\r' => { () });
    pred!(anything: (usize, char) => () = |_c| _c.1 != '\n' && _c.1 != '\r' => { () });
    seq!(comment: (usize, char) => I = (_, '#'), * anything, end_line, { I::Junk });

    alt!(main: (usize, char) => I = whitespace | comment);

    main(input)
});

group!(lower_symbol: (usize, char) => I = |input| {
    pred!(init_lower_symbol_char: (usize, char) = |c| c.1.is_lowercase() || c.1 == '_');
    pred!(rest_lower_symbol_char: (usize, char) = |c| c.1.is_alphanumeric() || c.1 == '_');
    alt!(both: (usize, char) = init_lower_symbol_char | rest_lower_symbol_char );
    seq!(main: (usize, char) => I = init <= init_lower_symbol_char, rs <= * both, {
        let start = init.0;
        let end = match rs.last() {
            Some(x) => x.0,
            None => init.0,
        };
        let meta = TMeta { start, end };
        I::T(Token::LowerSymbol(meta, format!( "{}{}", init.1, rs.into_iter().map(|x| x.1).collect::<String>())))
    } );

    main(input)
});

group!(upper_symbol: (usize, char) => I = |input| { 
    pred!(init_upper_symbol_char: (usize, char) = |c| c.1.is_uppercase());
    pred!(rest_upper_symbol_char: (usize, char) = |c| c.1.is_alphanumeric());
    alt!(both: (usize, char) = init_upper_symbol_char | rest_upper_symbol_char );
    seq!(main: (usize, char) => I = init <= init_upper_symbol_char, rs <= * both, {
        let start = init.0;
        let end = match rs.last() {
            Some(x) => x.0,
            None => init.0,
        };
        let meta = TMeta { start, end };
        I::T(Token::UpperSymbol(meta, format!( "{}{}", init.1, rs.into_iter().map(|x| x.1).collect::<String>())))
    } );

    main(input)
});

group!(string: (usize, char) => I = |input| {
    seq!(n: (usize, char) => char = (_, 'n'), { '\n' });
    seq!(r: (usize, char) => char = (_, 'r'), { '\r' });
    seq!(t: (usize, char) => char = (_, 't'), { '\t' });
    seq!(slash: (usize, char) => char = (_, '\\'), { '\\' });
    seq!(zero: (usize, char) => char = (_, '0'), { '\0' });
    seq!(quote: (usize, char) => char = (_, '"'), { '"' });

    alt!(code: (usize, char) => char = n | r | t | slash | zero | quote);
    seq!(escape: (usize, char) => char = slash, c <= ! code, { c });

    pred!(any: (usize, char) => char = |c| c.1 != '"' => { c.1 });
    alt!(str_char: (usize, char) => char = escape
                                         | any  
                                         );

    seq!(main: (usize, char) => I = _1 <= (_, '"'), sc <= * str_char, _2 <= (_, '"'), {
        let meta = TMeta { start: _1.0, end: _2.0 };
        I::T(Token::String(meta, sc.into_iter().collect::<String>()))
    });

    main(input)
});

group!(number: (usize, char) => I = |input| { 
    fn m<T : Into<String>>(input : Option<(usize, T)>) -> String {
        match input { 
            Some((_, x)) => x.into(),
            None => "".into()
        }
    }

    pred!(digit: (usize, char) = |c| c.1.is_digit(10));

    seq!(decimal: (usize, char) => (usize, String) = (_, '.'), d <= ! digit, ds <= * digit, {
        let end = match ds.last() {
            Some(x) => x.0,
            None => d.0,
        };
        (end, format!("{}{}", d.1, ds.into_iter().map(|x| x.1).collect::<String>()))
    });

    seq!(sci_not: (usize, char) => (usize, String) = (_, 'e') | (_, 'E')
                                                   , sign <= ? (_, '+') | (_, '-')
                                                   , d <= ! digit
                                                   , ds <= * digit, {
        let end = match ds.last() {
            Some(x) => x.0,
            None => d.0,
        };

        (end, format!( "e{}{}{}"
                     , m(sign)
                     , d.1
                     , ds.into_iter().map(|x| x.1).collect::<String>()))
    });

    seq!(main: (usize, char) => I = sign <= ? (_, '+') | (_, '-')
                                  , d <= digit
                                  , ds <= * digit
                                  , maybe_decimal <= ? decimal
                                  , maybe_sci_not <= ? sci_not, {
        let start = match sign {
            Some(x) => x.0,
            None => d.0,
        };
        let end = {
            let mut ret = d.0;
            match ds.last() {
                Some(x) => ret = x.0,
                None => { },
            }
            match &maybe_decimal {
                Some(x) => ret = x.0,
                None => { },
            }
            match &maybe_sci_not {
                Some(x) => ret = x.0,
                None => { },
            }
            ret
        };
        let meta = TMeta { start, end };
        let dot = match maybe_decimal {
            Some(_) => ".",
            None => "",
        };
        let n = format!("{}{}{}{}{}{}"
                       , m(sign)
                       , d.1
                       , ds.into_iter().map(|x| x.1).collect::<String>()
                       , dot
                       , m(maybe_decimal)
                       , m(maybe_sci_not));
        let ret = n.parse::<f64>().expect("allowed number string that rust fails to parse with parse::<f64>()");
        I::T(Token::Number(meta, ret))
    });

    main(input)
});

group!(punctuation: (usize, char) => I = |input| {
    fn m(x : (usize, char)) -> TMeta {
        TMeta { start: x.0, end: x.0 }
    }
    seq!(l_angle: (usize, char) => I = p <= (_, '<'), { I::T(Token::LAngle(m(p))) });
    seq!(r_angle: (usize, char) => I = p <= (_, '>'), { I::T(Token::RAngle(m(p))) });
    seq!(l_paren: (usize, char) => I = p <= (_, '('), { I::T(Token::LParen(m(p))) });
    seq!(r_paren: (usize, char) => I = p <= (_, ')'), { I::T(Token::RParen(m(p))) });
    seq!(l_curl: (usize, char) => I = p <= (_, '{'), { I::T(Token::LCurl(m(p))) });
    seq!(r_curl: (usize, char) => I = p <= (_, '}'), { I::T(Token::RCurl(m(p))) });
    seq!(colon: (usize, char) => I = p <= (_, ':'), { I::T(Token::Colon(m(p))) });
    seq!(dot: (usize, char) => I = p <= (_, '.'), { I::T(Token::Dot(m(p))) });

    alt!(single: (usize, char) => I = l_paren
                                    | r_paren
                                    | l_curl
                                    | r_curl
                                    | colon
                                    | dot
                                    | l_angle
                                    | r_angle
                                    );

    seq!(single_left_arrow: (usize, char) => I = _1 <= (_, '<'), _2 <= (_, '-'), {
        I::T(Token::SLArrow(TMeta { start: _1.0, end: _2.0 }))
    });
    seq!(double_left_arrow: (usize, char) => I = _1 <= (_, '<'), _2 <= (_, '='), {
        I::T(Token::DLArrow(TMeta { start: _1.0, end: _2.0 }))
    });
    seq!(single_right_arrow: (usize, char) => I = _1 <= (_, '-'), _2 <= (_, '>'), {
        I::T(Token::SRArrow(TMeta { start: _1.0, end: _2.0 }))
    });
    seq!(double_right_arrow: (usize, char) => I = _1 <= (_, '='), _2 <= (_, '>'), {
        I::T(Token::DRArrow(TMeta { start: _1.0, end: _2.0 }))
    });
    alt!(main: (usize, char) => I = single_left_arrow
                                  | double_left_arrow
                                  | single_right_arrow
                                  | double_right_arrow
                                  | single );

    main(input)
});


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_single_right_arrow() -> Result<(), MatchError> {
        let input = r#"->"#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end) = match &output[0] {
            I::T(Token::SRArrow(m)) => (m.start, m.end),
            x => panic!("found unexpected: {:?}", x),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, 1 );
        
        Ok(())
    }

    #[test]
    fn should_parse_right_angle() -> Result<(), MatchError> {
        let input = r#">"#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end) = match &output[0] {
            I::T(Token::RAngle(m)) => (m.start, m.end),
            _ => panic!("not punctuation"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, 0 );
        
        Ok(())
    }

    #[test]
    fn should_parse_left_angle() -> Result<(), MatchError> {
        let input = r#"<"#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end) = match &output[0] {
            I::T(Token::LAngle(m)) => (m.start, m.end),
            _ => panic!("not punctuation"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, 0 );
        
        Ok(())
    }

    #[test]
    fn should_parse_double_left_arrow() -> Result<(), MatchError> {
        let input = r#"<="#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end) = match &output[0] {
            I::T(Token::DLArrow(m)) => (m.start, m.end),
            _ => panic!("not punctuation"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, 1 );
        
        Ok(())
    }

    #[test]
    fn should_parse_single_left_arrow() -> Result<(), MatchError> {
        let input = r#"<-"#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end) = match &output[0] {
            I::T(Token::SLArrow(m)) => (m.start, m.end),
            _ => panic!("not punctuation"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, 1 );
        
        Ok(())
    }

    #[test]
    fn should_parse_comment() -> Result<(), MatchError> {
        let input = r#"#this is a comment
        blah"#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 3 );

        assert!( matches!( output[0], I::Junk ) );
        
        Ok(())
    }

    #[test]
    fn should_parse_whitespace() -> Result<(), MatchError> {
        let input = "      \n\t\rfalse";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 2 );

        assert!( matches!( output[0], I::Junk ) );

        Ok(())
    }

    macro_rules! string_test {
        ($name:ident: $input:expr => $expected:expr) => {
            #[test]
            fn $name() -> Result<(), MatchError> {
                let output = internal_tokenize($input)?;

                assert_eq!( output.len(), 1 );

                let (start, end, value) = match &output[0] {
                    I::T(Token::String(m, n)) => (m.start, m.end, n.clone()),
                    _ => panic!("not string"),
                };

                assert_eq!( start, 0 );
                assert_eq!( end, $input.len() - 1 );
                assert_eq!( value, $expected );
                Ok(())
            }
        };
    }

    string_test!(should_parse_string: r#""string input""# => "string input");
    string_test!(should_parse_string_with_slash_n: r#""string \n input""# => "string \n input");
    string_test!(should_parse_string_with_slash_r: r#""string \r input""# => "string \r input");
    string_test!(should_parse_string_with_slash_zero: r#""string \0 input""# => "string \0 input");
    string_test!(should_parser_string_with_slash_t: r#""string \t input""# => "string \t input");
    string_test!(should_parse_string_with_slash_slash: r#""string \\ input""# => "string \\ input");
    string_test!(should_parse_string_with_slash_quote: r#""string \" input""# => "string \" input");

    macro_rules! number_test {
        ($name:ident: $input:expr => $expected:expr) => {
            #[test]
            fn $name() -> Result<(), MatchError> {
                let output = internal_tokenize($input)?;

                assert_eq!( output.len(), 1 );

                let (start, end, value) = match &output[0] {
                    I::T(Token::Number(m, n)) => (m.start, m.end, *n),
                    _ => panic!("not number"),
                };

                assert_eq!( start, 0 );
                assert_eq!( end, $input.len() - 1 );
                assert_eq!( value, $expected );
                Ok(())
            }
        };
    }

    number_test!(should_parse_zero: "0" => 0.0);
    number_test!(should_parse_zero_point_zero: "0.0" => 0.0);
    number_test!(should_parse_negative: "-1" => -1.0);
    number_test!(should_parse_plus: "+1" => 1.0);
    number_test!(should_parse_sci_not_big_e: "1E1" => 1E1);
    number_test!(should_parse_sci_not_little_e: "1e1" => 1e1);
    number_test!(should_parse_plus_one: "+1.0" => 1.0);
    number_test!(should_parse_neg_one: "-1.0" => -1.0);
    number_test!(should_parse_sci_not_plus_big_e: "1E+1" => 1E+1);
    number_test!(should_parse_sci_not_plus_little_e: "1e+1" => 1e+1);
    number_test!(should_parse_decimal: "1234.5678" => 1234.5678);
    number_test!(should_parse_decimal_with_sci_not_neg_big_e: "1234.5678E-90" => 1234.5678E-90);
    number_test!(should_parse_decimal_with_sci_not_neg_little_e: "1234.5678e-90" => 1234.5678e-90);
    number_test!(should_parse_decimal_with_sci_not_neg_little_e_901: "1234.5678e-901" => 1234.5678e-901);
    number_test!(should_parse_number: "1234" => 1234.0);

    #[test]
    fn should_parse_boolean_starting_lower_symbol() -> Result<(), MatchError> {
        let input = "false_";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end, name) = match &output[0] {
            I::T(Token::LowerSymbol(m, n)) => (m.start, m.end, n.clone()),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, input.len() - 1 );
        assert_eq!( name, "false_" );

        Ok(())
    }

    #[test]
    fn should_parse_lower_symbol() -> Result<(), MatchError> {
        let input = "lower_symbol";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end, name) = match &output[0] {
            I::T(Token::LowerSymbol(m, n)) => (m.start, m.end, n.clone()),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, input.len() - 1 );
        assert_eq!( name, "lower_symbol" );

        Ok(())
    }

    #[test]
    fn should_parse_single_lower_symbol() -> Result<(), MatchError> {
        let input = "l";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end, name) = match &output[0] {
            I::T(Token::LowerSymbol(m, n)) => (m.start, m.end, n.clone()),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, input.len() - 1 );
        assert_eq!( name, "l" );

        Ok(())
    }

    #[test]
    fn should_parse_upper_symbol() -> Result<(), MatchError> {
        let input = "UpperSymbol";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end, name) = match &output[0] {
            I::T(Token::UpperSymbol(m, n)) => (m.start, m.end, n.clone()),
            _ => panic!("not upper symbol"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, input.len() - 1 );
        assert_eq!( name, "UpperSymbol" );

        Ok(())
    }

    #[test]
    fn should_parse_single_upper_symbol() -> Result<(), MatchError> {
        let input = "U";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );

        let (start, end, name) = match &output[0] {
            I::T(Token::UpperSymbol(m, n)) => (m.start, m.end, n.clone()),
            _ => panic!("not upper symbol"),
        };

        assert_eq!( start, 0 );
        assert_eq!( end, input.len() - 1 );
        assert_eq!( name, "U" );

        Ok(())
    }
}
