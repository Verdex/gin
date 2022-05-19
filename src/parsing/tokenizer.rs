
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use super::data::{Token, TMeta};

pub fn tokenize( input : &str ) -> Result<Vec<Token>, String> {
    Err("TODO".into())
}

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
    pred!(ws: (usize, char) => () = |c| c.1.is_whitespace() => { () });
    seq!(whitespace: (usize, char) => I = ws, * ws, { I::Junk });

    pred!(end_line: (usize, char) => () = |c| c.1 == '\n' || c.1 == '\r' => { () });
    pred!(anything: (usize, char) => () = |c| c.1 != '\n' && c.1 != '\r' => { () });
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

/*group!(number: (usize, char) => I = |input| { 
    pred!(digit: (usize, char) => char = |c| c.1.is_digit(10) => { c.1 });

    seq!(science: (usize, char) => String = 'e'|'E', ms <= ? '+'|'-', init <= ! digit, ds <= * digit, {
        match ms {
            Some(x) => format!("e{}{}{}", x, init, ds.into_iter().collect::<String>()),
            None => format!("e{}{}", init, ds.into_iter().collect::<String>()),
        }
    } );

    alt!(initial: char => char = sign | digit );

    seq!(main<'a>: char => String = init <= initial, whole <= digits, d <= dot, fractional <= digits, s <= science, {
        let ret = format!("{}{}", init, whole.into_iter().collect::<String>());
        let ret = match d { 
            Some(_) => format!("{}.{}", ret, fractional.into_iter().collect::<String>()),
            None => ret,
        };
        match s {
            Some(s) => format!("{}{}", ret, s),
            None => ret,
        }
    });

    match main(input) {
        Ok(Success { item, start, end }) => {
            let ret = item.parse::<f64>().expect("allowed number string that rust fails to parse with parse::<f64>()");
            Ok(Success { item: InternalToken::Number(ret), start, end })
        },
        Err(e) => Err(e),
    }
});*/

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn should_parse_string() -> Result<(), MatchError> {
        fn t(input : &str, expected : &str) -> Result<(), MatchError> {
            let output = internal_tokenize(input)?;

            assert_eq!( output.len(), 1 );

            let (start, end, value) = match &output[0] {
                I::T(Token::String(m, n)) => (m.start, m.end, n.clone()),
                _ => panic!("not string"),
            };

            assert_eq!( start, 0 );
            assert_eq!( end, input.len() - 1 );
            assert_eq!( value, expected );
            Ok(())
        }

        t(r#""string input""#, "string input")?;
        t(r#""string \n input""#, "string \n input")?;
        t(r#""string \r input""#, "string \r input")?;
        t(r#""string \0 input""#, "string \0 input")?;
        t(r#""string \t input""#, "string \t input")?;
        t(r#""string \\ input""#, "string \\ input")?;
        t(r#""string \" input""#, "string \" input")?;

        Ok(())
    }

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
