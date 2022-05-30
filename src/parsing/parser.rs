
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{TMeta, Token, AMeta, Type};

pub fn parse(tokens : Vec<Token>) -> Result<(), String> {

    Err("TODO".into())
}

group!(parse_type<'a>: &'a Token => Type = |input| {
    seq!(concrete<'a>: &'a Token => Type = name <= Token::UpperSymbol(_, _), {
        let ameta = AMeta { token_meta: vec![name.meta()] };
        Type::Concrete(ameta, name.symbol_name())
    });

    seq!(generic<'a>: &'a Token => Type = name <= Token::LowerSymbol(_, _), {
        let ameta = AMeta { token_meta: vec![name.meta()] };
        Type::Generic(ameta, name.symbol_name())
    });

    seq!(index<'a>: &'a Token => Type = name <= Token::UpperSymbol(_, _)
                                      , l <= Token::LAngle(_) 
                                      , indexer <= ! main
                                      , r <= ! Token::RAngle(_) 
                                      , {
        let ameta = AMeta { token_meta: vec![name.meta(), l.meta(), r.meta()]};
        Type::Index(ameta, name.symbol_name(), Box::new(indexer))
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
            let ameta = AMeta { token_meta: vec![] };
            Type::Arrow { meta: ameta, src: Box::new(t), dest: Box::new(r) }
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

    test_type!(should_parse_generic: "a" => Type::Generic(_, n) => { assert_eq!(n, "a"); });
    test_type!(should_parse_concrete: "A" => Type::Concrete(_, n) => { assert_eq!(n, "A"); });
    test_type!(should_parse_paren_generic: "(a)" => Type::Generic(_, n) => { assert_eq!(n, "a"); });
    test_type!(should_parse_paren_concrete: "(A)" => Type::Concrete(_, n) => { assert_eq!(n, "A"); });
    test_type!(should_parse_index: "Blah<a>" => Type::Index(_, n, g) => {
        assert_eq!( n, "Blah" );
        if let Type::Generic(_, g_n) = *g {
            assert_eq!( g_n, "a" );
        }
        else {
            panic!("expected generic as indexer");
        }
    });
    test_type!(should_parse_simple_arrow: "a -> b" => Type::Arrow { src, dest, .. } => {
        if let (Type::Generic(_, src), Type::Generic(_, dest)) = (*src, *dest) {
            assert_eq!(src, "a");
            assert_eq!(dest, "b");
        }
        else {
            panic!("src and dest should be generic types");
        }
    });
    test_type!(should_parse_arrow_arrow: "a -> b -> c" => Type::Arrow { src, dest, .. } => {
        if let Type::Generic(_, src) = *src {
            assert_eq!(src, "a");
        }
        else {
            panic!("Arrow source incorrect");
        }
        if let Type::Arrow { src, dest, .. } = *dest {
            if let (Type::Generic(_, dest_src), Type::Generic(_, dest_dest)) = (*src, *dest) {
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
    test_type!(should_parse_arrow_param: "(a -> b) -> c" => Type::Arrow { src, dest, .. } => {
        if let Type::Arrow { src, dest, .. } = *src {
            if let (Type::Generic(_, src_src), Type::Generic(_, src_dest)) = (*src, *dest) {
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
        if let Type::Generic(_, dest) = *dest {
            assert_eq!(dest, "c");
        }
        else {
            panic!("Arrow dest incorrect");
        }
    });
}

