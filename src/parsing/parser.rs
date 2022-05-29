
use crate::{alt, group, pred, seq, cases};
use crate::array_pattern::MatchError;
use crate::data::{TMeta, Token, AMeta, Type};

pub fn parse(tokens : Vec<Token>) -> Result<(), String> {

    Err("TODO".into())
}

group!(parse_type: Token => Type = |input| {
    seq!(concrete: Token => Type = name <= Token::UpperSymbol(_, _), {
        if let Token::UpperSymbol(meta, name) = name {
            let ameta = AMeta { token_meta: vec![meta] };
            Type::ConcreteType(ameta, name)
        }
        else {
            panic!("Expected UpperSymbol");
        }
    });

    seq!(generic: Token => Type = name <= Token::LowerSymbol(_, _), {
        if let Token::LowerSymbol(meta, name) = name {
            let ameta = AMeta { token_meta: vec![meta] };
            Type::ConcreteType(ameta, name)
        }
        else {
            panic!("Expected LowerSymbol");
        }
    });

    seq!(index: Token => Type = name <= Token::UpperSymbol(_, _)
                              , l <= Token::LAngle(_) 
                              , indexee <= ! main
                              , r <= ! Token::RAngle(_) 
                              , {

        if let ( Token::UpperSymbol(name_meta, name)
               , Token::LAngle(l_meta)
               , Token::RAngle(r_meta)
               ) = (name, l, r) {
            let ameta = AMeta { token_meta: vec![name_meta, l_meta, r_meta]};
            Type::Index(ameta, name, Box::new(indexee))
        }
        else {
            panic!("Unexpected cases");
        }
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

