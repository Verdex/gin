
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
    } );

    alt!(main: Token => Type = index
                             | generic 
                             | concrete
                             );

    main(input)
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should() {

    }
}

