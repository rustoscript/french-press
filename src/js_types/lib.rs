use String;

pub enum JsPrim {
    Undef,
    Null,
    Bool(bool),
    Num(f64),
    Symbol(std::String),
    JsStr(std::String),
}

pub struct JsObj;
