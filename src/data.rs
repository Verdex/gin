
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
    Punct(TMeta, String),
}

#[derive(Debug)]
pub struct AMeta {
    token_meta : Vec<TMeta>,
}

#[derive(Debug)]
pub enum Type {
    ConcreteType(AMeta, String),
    GenericType(AMeta, String),
    IndexType(AMeta, String, Box<Type>),
    ArrowType{ meta: AMeta, src: Box<Type>, dest: Box<Type> },
}