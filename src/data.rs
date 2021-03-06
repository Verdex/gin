
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
pub enum Type {
    Concrete(String),
    Generic(String),
    Index(String, Box<Type>),
    Arrow{ src: Box<Type>, dest: Box<Type> },
}

#[derive(Debug)]
pub enum Expr {
    //Let { name : String }
}

#[derive(Debug)]
pub enum ConsCase {
    Position { name : String, params : Vec<Type> },
}

#[derive(Debug)]
pub enum Ast {
    ConsDef { name : String, type_params : Vec<String>, cons : Vec<ConsCase> },
    LetDef { name : String, type_params : Vec<String>, t : Type, expr : Expr },
}
