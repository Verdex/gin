
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{ TMeta 
                 , Token
                 , Type
                 , ConsCase
                 , Ast
                 };

pub fn parse(input : &str, tokens : Vec<Token>) -> Result<Vec<Ast>, String> {
    match internal_parse(&tokens) {
        Ok(asts) => Ok(asts),
        Err(MatchError::ErrorEndOfFile | MatchError::FatalEndOfFile) =>
            Err("Encountered unexpected end of file while parsing.".into()),
        Err(MatchError::Error(i) | MatchError::Fatal(i)) => {
            let TMeta { start, end } = tokens[i].meta();
            let error_reporter::ErrorReport { line, column, display } = error_reporter::report(input, start, end);
            let ret = format!("Encountered parsing error at line {line} and column {column}:\n\n{display}");
            Err(ret)
        },
    }
}

fn internal_parse(tokens : &Vec<Token>) -> Result<Vec<Ast>, MatchError> {
    let mut x = tokens.iter().enumerate();

    alt!( ast<'a>: &'a Token => Ast = parse_cons_def);

    let mut ret = vec![];
    loop {
        match ast(&mut x) {
            Ok(t) => ret.push(t),
            Err(MatchError::ErrorEndOfFile) => break,
            Err(e) => return Err(e),
        }
    }

    Ok(ret)
}

group!(parse_cons_def<'a>: &'a Token => Ast = |input| {
    seq!(comma_type<'a>: &'a Token => Type = Token::Comma(_), t <= ! parse_type, { t });
    seq!(type_list<'a>: &'a Token => Vec<Type> = Token::LParen(_)
                                               , _1 <= ! parse_type
                                               , r <= * comma_type
                                               , ! Token::RParen(_)
                                               , {
        let mut rest = r;
        rest.insert(0, _1);
        rest
    });
    seq!(comma_sym<'a>: &'a Token => String = Token::Comma(_), s <= ! Token::LowerSymbol(_, _), {
        s.symbol_name()
    });
    seq!(generic_list<'a>: &'a Token => Vec<String> = Token::LAngle(_)
                                                    , _1 <= ! Token::LowerSymbol(_, _)
                                                    , r <= * comma_sym
                                                    , ! Token::RAngle(_)
                                                    , {
        let mut rest = r;
        rest.insert(0, _1.symbol_name());
        rest
    });
    seq!(cons<'a>: &'a Token => ConsCase = name <= Token::UpperSymbol(_, _), params <= ? type_list, {
        let params = match params {
            Some(v) => v,
            None => vec![],
        };
        ConsCase::Position { name: name.symbol_name(), params }
    });
    seq!(cons_comma<'a>: &'a Token => ConsCase = c <= cons, Token::Comma(_), { c });
    seq!(cons_list<'a>: &'a Token => Vec<ConsCase> = Token::LCurl(_)
                                                   , cs <= * cons_comma
                                                   , l <= ? cons
                                                   , ! Token::RCurl(_)
                                                   , {
        let mut cases = cs;
        match l {
            Some(case) => cases.push(case),
            None => { },
        }
        cases 
    });
    pred!(type_keyword<'a>: &'a Token => () = |x| matches!(x, Token::LowerSymbol(_, _)) && x.symbol_name() == "type" => { () });
    seq!(main<'a>: &'a Token => Ast = type_keyword
                                        , gs <= ? generic_list
                                        , name <= ! Token::UpperSymbol(_, _)
                                        , cs <= ? cons_list
                                        , {
        let type_params = match gs {
            Some(v) => v,
            None => vec![],
        };
        let cs = match cs { 
            Some(v) => v,
            None => vec![],
        };
        Ast::ConsDef{ name: name.symbol_name(), type_params, cons: cs }
    });
    main(input)
});

group!(parse_type<'a>: &'a Token => Type = |input| {
    seq!(concrete<'a>: &'a Token => Type = name <= Token::UpperSymbol(_, _), {
        Type::Concrete(name.symbol_name())
    });

    seq!(generic<'a>: &'a Token => Type = name <= Token::LowerSymbol(_, _), {
        Type::Generic(name.symbol_name())
    });

    seq!(index<'a>: &'a Token => Type = name <= Token::UpperSymbol(_, _)
                                      , l <= Token::LAngle(_) 
                                      , indexer <= ! main
                                      , r <= ! Token::RAngle(_) 
                                      , {
        Type::Index(name.symbol_name(), Box::new(indexer))
    });

    seq!(paren<'a>: &'a Token => Type = Token::LParen(_)
                                      , t <= main
                                      , ! Token::RParen(_)
                                      , {
        t
    });

    alt!(atomic<'a>: &'a Token => Type = paren
                                       | index
                                       | generic 
                                       | concrete
                                       );

    seq!(rest<'a>: &'a Token => Type = Token::SRArrow(_)
                                     , t <= ! main
                                     , {
        t
    });
    seq!(main<'a>: &'a Token => Type = t <= atomic
                                     , r <= ? rest
                                     , {
        if let Some(r) = r {
            Type::Arrow { src: Box::new(t), dest: Box::new(r) }
        }
        else {
            t
        }
    });

    main(input)
});

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_first_parse {
        ($name:ident: $input:expr => $expected:pat => $x:block) => {
            #[test]
            fn $name() -> Result<(), MatchError> {
                use super::super::tokenizer::tokenize;
                if let Ok(tokens) = tokenize($input) {
                    let mut output = internal_parse(&tokens)?;

                    assert_eq!( output.len(), 1 );

                    if let Some($expected) = output.pop() {
                        $x
                    }
                    else {
                        panic!("instead of expected pattern found: {:?}\nfrom tokens: {:?}", output, tokens);
                    }
                    Ok(())
                }
                else {
                    panic!( "tokenize failed in test parse" );
                }
            }
        };
    }

    macro_rules! test_type {
        ($name:ident: $input:expr => $expected:pat => $x:block) => {
            #[test]
            fn $name() -> Result<(), MatchError> {
                use super::super::tokenizer::tokenize;
                if let Ok(tokens) = tokenize($input) {
                    let output = parse_type(&mut tokens.iter().enumerate())?;
                    if let $expected = output {
                        $x
                    }
                    else {
                        panic!("instead of expected pattern found: {:?}\nfrom tokens: {:?}", output, tokens);
                    }
                    Ok(())
                }
                else {
                    panic!( "tokenize failed in test type" );
                }
            }
        };
    }

    test_type!(should_parse_generic: "a" => Type::Generic(n) => { assert_eq!(n, "a"); });
    test_type!(should_parse_concrete: "A" => Type::Concrete(n) => { assert_eq!(n, "A"); });
    test_type!(should_parse_paren_generic: "(a)" => Type::Generic(n) => { assert_eq!(n, "a"); });
    test_type!(should_parse_paren_concrete: "(A)" => Type::Concrete(n) => { assert_eq!(n, "A"); });
    test_type!(should_parse_index: "Blah<a>" => Type::Index(n, g) => {
        assert_eq!( n, "Blah" );
        if let Type::Generic(g_n) = *g {
            assert_eq!( g_n, "a" );
        }
        else {
            panic!("expected generic as indexer");
        }
    });
    test_type!(should_parse_simple_arrow: "a -> b" => Type::Arrow { src, dest } => {
        if let (Type::Generic(src), Type::Generic(dest)) = (*src, *dest) {
            assert_eq!(src, "a");
            assert_eq!(dest, "b");
        }
        else {
            panic!("src and dest should be generic types");
        }
    });
    test_type!(should_parse_arrow_arrow: "a -> b -> c" => Type::Arrow { src, dest } => {
        if let Type::Generic(src) = *src {
            assert_eq!(src, "a");
        }
        else {
            panic!("Arrow source incorrect");
        }
        if let Type::Arrow { src, dest, .. } = *dest {
            if let (Type::Generic(dest_src), Type::Generic(dest_dest)) = (*src, *dest) {
                assert_eq!( dest_src, "b");
                assert_eq!( dest_dest, "c");
            }
            else {
                panic!("Arrow Dest incorrect");
            }
        }
        else {
            panic!("Dest should be arrow type");
        }
    });
    test_type!(should_parse_arrow_param: "(a -> b) -> c" => Type::Arrow { src, dest } => {
        if let Type::Arrow { src, dest } = *src {
            if let (Type::Generic(src_src), Type::Generic(src_dest)) = (*src, *dest) {
                assert_eq!( src_src, "a");
                assert_eq!( src_dest, "b");
            }
            else {
                panic!("Arrow source incorrect");
            }
        }
        else {
            panic!("source should be arrow type");
        }
        if let Type::Generic(dest) = *dest {
            assert_eq!(dest, "c");
        }
        else {
            panic!("Arrow dest incorrect");
        }
    });
    test_type!(should_parse_arrow_in_index: "Blah<a -> b>" => Type::Index(_, arrow) => { 
        assert!( matches!( *arrow, Type::Arrow{ .. } ) );
    });
    test_type!(should_parse_index_in_arrow: "Blah<a> -> Blah<b>" => Type::Arrow { src, dest } => {
        assert!( matches!( *src, Type::Index(_, _) ) );
        assert!( matches!( *dest, Type::Index(_, _) ) );
    });

    test_first_parse!(should_parse_enum_style_type: r#"
        type Name { 
            First,
            Second,
            Third
        }"# 
        
        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 0 );
            assert_eq!( cons.len(), 3 );
    });

    test_first_parse!(should_parse_tuple_case_type: r#"
        type Name { 
            First,
            Second(One, Two, Three),
            Third(One)
        }"# 
        
        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 0 );
            assert_eq!( cons.len(), 3 );
    });

    test_first_parse!(should_parse_single_generic_case_type: r#"
        type<a> Name { 
            First,
            Second(a, Two, Three),
            Third(One)
        }"# 
        
        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 1 );
            assert_eq!( type_params[0], "a" );
            assert_eq!( cons.len(), 3 );
    });

    test_first_parse!(should_parse_generic_case_type: r#"
        type<a, b, c> Name { 
            First,
            Second(a, b, c),
            Third(One)
        }"# 
        
        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 3 );
            assert_eq!( type_params[0], "a" );
            assert_eq!( type_params[1], "b" );
            assert_eq!( type_params[2], "c" );
            assert_eq!( cons.len(), 3 );
    });

    test_first_parse!(should_parse_type_trailing_comma: r#"
        type<a, b, c> Name { 
            First,
            Second(a, b, c),
            Third(One),
        }"# 
        
        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 3 );
            assert_eq!( type_params[0], "a" );
            assert_eq!( type_params[1], "b" );
            assert_eq!( type_params[2], "c" );
            assert_eq!( cons.len(), 3 );
    });

    test_first_parse!(should_parse_single_case_type: r#"
        type Name { First }"#

        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 0 );
            assert_eq!( cons.len(), 1 );
        });

    test_first_parse!(should_parse_empty_type: r#"
        type Name { }"#

        => Ast::ConsDef { name, type_params, cons }

        => {
            assert_eq!( name, "Name" );
            assert_eq!( type_params.len(), 0 );
            assert_eq!( cons.len(), 0 );
        });  
}

