
#[derive(Debug)]
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
    RParen(TMeta),
    LParen(TMeta),
    LAngle(TMeta),
    RAngle(TMeta),
    SLArrow(TMeta),
    SRArrow(TMeta),
    DLArrow(TMeta),
    DRArrow(TMeta),
    Colon(TMeta),
    Dot(TMeta),
}

#[derive(Debug)]
pub struct AMeta {
    pub token_meta : Vec<TMeta>,
}

#[derive(Debug)]
pub enum Type {
    Concrete(AMeta, String),
    Generic(AMeta, String),
    Index(AMeta, String, Box<Type>),
    Arrow{ meta: AMeta, src: Box<Type>, dest: Box<Type> },
}