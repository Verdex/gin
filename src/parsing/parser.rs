
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


    // TODO:  need to fix left recursion issue here
    seq!(arrow: Token => Type = src <= main
                              , a <= Token::SRArrow(_)
                              , dest <= ! main
                              , {
        let ameta = AMeta { token_meta: vec![a.meta()] };
        let src = Box::new(src);
        let dest = Box::new(dest);
        Type::Arrow{ src, dest }
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

    seq!(main: Token => Type = );

    main(input)
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should() {

    }
}

