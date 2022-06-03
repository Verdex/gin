
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{ TMeta 
                 , Token
                 , AMeta
                 , Type
                 , ConsCase
                 , Ast
                 };

pub fn parse(tokens : Vec<Token>) -> Result<Vec<Ast>, String> {
    Err("TODO".into())
}

fn internal_parse(tokens : Vec<Token>) -> Result<Vec<Ast>, MatchError> {
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
        let meta = AMeta { token_meta : vec![] };
        let params = match params {
            Some(v) => v,
            None => vec![],
        };
        ConsCase::Position { meta, name: name.symbol_name(), params }
    });
    seq!(comma_cons<'a>: &'a Token => ConsCase = Token::Comma(_), c <= ! cons, { c });
    seq!(cons_list<'a>: &'a Token => Vec<ConsCase> = Token::LCurl(_)
                                                   , _1 <= ! cons
                                                   , r <= * comma_cons
                                                   , ! Token::RCurl(_)
                                                   , {
        let mut rest = r;
        rest.insert(0, _1);
        rest
    });
    pred!(type_keyword<'a>: &'a Token => () = |x| matches!(x, Token::LowerSymbol(_, _)) && x.symbol_name() == "type" => { () });
    seq!(main<'a>: &'a Token => Ast = type_keyword
                                        , name <= ! Token::UpperSymbol(_, _)
                                        , gs <= ? generic_list
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
    /*type Blah[<a,+>] {
        UpperSymbol,
        UpperSymbol(Type,+),
    }*/
    main(input)
});

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
    test_type!(should_parse_arrow_in_index: "Blah<a -> b>" => Type::Index(_, _, arrow) => { 
        assert!( matches!( *arrow, Type::Arrow{ .. } ) );
    });
    test_type!(should_parse_index_in_arrow: "Blah<a> -> Blah<b>" => Type::Arrow { src, dest, .. } => {
        assert!( matches!( *src, Type::Index(_, _, _) ) );
        assert!( matches!( *dest, Type::Index(_, _, _) ) );
    });
}

