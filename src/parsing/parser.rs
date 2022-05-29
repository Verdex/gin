
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{TMeta, Token, AMeta, Type};

pub fn parse(tokens : Vec<Token>) -> Result<(), String> {

    Err("TODO".into())
}

group!(parse_type: Token => Type = |input| {
    seq!(concrete: Token => Type = name <= Token::UpperSymbol(_, _), {
        let ameta = AMeta { token_meta: vec![name.meta()] };
        Type::Concrete(ameta, name.symbol_name())
    });

    seq!(generic: Token => Type = name <= Token::LowerSymbol(_, _), {
        let ameta = AMeta { token_meta: vec![name.meta()] };
        Type::Generic(ameta, name.symbol_name())
    });

    seq!(index: Token => Type = name <= Token::UpperSymbol(_, _)
                              , l <= Token::LAngle(_) 
                              , indexee <= ! main
                              , r <= ! Token::RAngle(_) 
                              , {
        let ameta = AMeta { token_meta: vec![name.meta(), l.meta(), r.meta()]};
        Type::Index(ameta, name.symbol_name(), Box::new(indexee))
    });

    seq!(paren: Token => Type = Token::LParen(_)
                              , t <= main
                              , ! Token::RParen(_)
                              , {
        t
    });

    alt!(atomic: Token => Type = paren
                               | index
                               | generic 
                               | concrete
                               );

    seq!(rest: Token => Type = Token::SRArrow(_)
                             , t <= ! main
                             , {
        t
    });
    seq!(main: Token => Type = t <= atomic
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

    #[test]
    fn should() {

    }
}

