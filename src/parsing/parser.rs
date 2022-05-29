
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
        ($name:ident: $input:expr => $expected:pat) => {
            #[test]
            fn $name() -> Result<(), MatchError> {
                use super::super::tokenizer::tokenize;
                if let Ok(tokens) = tokenize($input) {
                    let output = parse_type(&mut tokens.iter().enumerate())?;
                    assert!( matches!( output, $expected ) );
                    Ok(())
                }
                else {
                    panic!( "tokenize failed in test type" );
                }
            }
        };
    }

    test_type!(blarg: "a" => Type::Generic(_, _));

}

