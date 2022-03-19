pub mod parser;

#[macro_use]
extern crate pest_derive;

pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Type {
    Bool, Int, Hex, String
}

#[derive(Debug)]
pub struct Config {
    /// The name of the config variable
    pub name: String,
    /// The type of the config variable
    pub ty: Option<Type>,
}


#[derive(Debug)]
pub enum Entry {
    Config(Config)
}

#[derive(Debug)]
pub struct KConfig {
    pub name: String,
    pub entries: Vec<Entry>,
}

