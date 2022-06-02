
#[derive(Debug, Clone, Copy)]
pub struct TMeta {
    pub start : usize,
    pub end : usize,
}

#[derive(Debug)]
pub enum Token {
    LowerSymbol(TMeta, String),
    UpperSymbol(TMeta, String),
    Number(TMeta, f64),
    String(TMeta, String),
    LParen(TMeta),
    RParen(TMeta),
    LCurl(TMeta),
    RCurl(TMeta),
    LAngle(TMeta),
    RAngle(TMeta),
    SLArrow(TMeta),
    SRArrow(TMeta),
    DLArrow(TMeta),
    DRArrow(TMeta),
    Colon(TMeta),
    Dot(TMeta),
    Comma(TMeta),
}

impl Token {
    pub fn meta(&self) -> TMeta {
        match self {
            Token::LowerSymbol(m, _) => *m,
            Token::UpperSymbol(m, _) => *m,
            Token::Number(m, _) => *m,
            Token::String(m, _) => *m,
            Token::LParen(m) => *m,
            Token::RParen(m) => *m,
            Token::LCurl(m) => *m,
            Token::RCurl(m) => *m,
            Token::LAngle(m) => *m,
            Token::RAngle(m) => *m,
            Token::SLArrow(m) => *m,
            Token::SRArrow(m) => *m,
            Token::DLArrow(m) => *m,
            Token::DRArrow(m) => *m,
            Token::Colon(m) => *m,
            Token::Dot(m) => *m,
            Token::Comma(m) => *m,
        }
    }

    pub fn symbol_name(&self) -> String {
        match self {
            Token::LowerSymbol(_, name) => name.into(),
            Token::UpperSymbol(_, name) => name.into(),
            _ => panic!("symbol_name called on non-symbol"),
        }
    }
}

#[derive(Debug)]
pub struct AMeta { // TODO do we actually need AMeta ? ... it's going to be something like lookup token index and then figure out the start and end from the meta on that
    pub token_meta : Vec<TMeta>,
}

#[derive(Debug)]
pub enum Type {
    Concrete(AMeta, String),
    Generic(AMeta, String),
    Index(AMeta, String, Box<Type>),
    Arrow{ meta: AMeta, src: Box<Type>, dest: Box<Type> },
}

#[derive(Debug)]
pub enum ConsCase {
    Position { meta : AMeta, name : String, params : Vec<Type> },
}

#[derive(Debug)]
pub struct ConsDef {
    pub meta : AMeta,
    pub name : String,
    pub type_params : Vec<String>,
    pub cons : Vec<ConsCase>,
}