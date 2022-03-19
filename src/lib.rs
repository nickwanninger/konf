pub mod parser;

pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;


use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    Bool, Int, Hex, String
}

impl Type {
    pub fn new(s: &str) -> Option<Self> {
        let t = match s {
            "bool" => Self::Bool,
            "int" => Self::Int,
            "hex" => Self::Hex,
            "string" => Self::String,
            _ => return None,
        };
        Some(t)
    }
}


impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Hex => "hex",
            Self::String => "string",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub struct Variable {
    /// The name of the config
    pub name: String,
    /// The type of the config
    pub ty: Option<Type>,
    /// A description of the config
    pub desc: Option<String>,
}

impl Variable {
    pub fn new(name: &str) -> Self {
        Variable {
            name: name.to_string(),
            ty: None,
            desc: None,
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "config {}", self.name)?;
        if let Some(t) = self.ty {
            write!(f, "  {t}")?;
            if let Some(d) = &self.desc {
                write!(f, " \"{d}\"")?;
            }
            
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Entry {
    Variable(Variable)
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Entry::Variable(v) => v.fmt(f)
        }
    }
}

#[derive(Debug)]
pub struct KConfig {
    pub name: String,
    pub entries: Vec<Entry>,
}

impl KConfig {
    pub fn source(&mut self, mut other: Self) {
        self.entries.append(&mut other.entries);
    }
}

impl fmt::Display for KConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "mainmenu \"{}\"", self.name)?;
        f.write_str("\n")?;
        for entry in &self.entries {
            entry.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}
