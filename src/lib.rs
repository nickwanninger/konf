pub mod parser;

pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;
use indexmap::IndexMap;
use regex::Regex;
use std::fmt;
use std::io::{self, prelude::*, BufReader};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    Bool,
    Int,
    Hex,
    String,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool), // y/n
    Int(i64),
    Hex(u64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool(true) => write!(f, "y")?,
            Self::Bool(false) => write!(f, "n")?,
            Self::Int(i) => write!(f, "{}", i)?,
            Self::Hex(i) => write!(f, "{:#x}", i)?,
            Self::String(s) => f.write_str(s)?,
        };
        Ok(())
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
    /// The current value. Inherits from `default`
    pub value: Option<Value>,
    /// The default value
    pub default: Option<Value>,
}

impl Variable {
    pub fn new(name: &str) -> Self {
        Variable {
            name: name.to_string(),
            ty: None,
            desc: None,
            value: None,
            default: None,
        }
    }
}

fn spaces(f: &mut fmt::Formatter, depth: i32) -> fmt::Result {
    for _i in 0..depth {
        write!(f, "    ")?;
    }
    Ok(())
}

impl Variable {
    fn pretty_format(&self, f: &mut fmt::Formatter, depth: i32) -> fmt::Result {
        spaces(f, depth)?;
        writeln!(f, "config {}", self.name)?;
        if let Some(t) = self.ty {
            spaces(f, depth + 1)?;
            write!(f, "{t}")?;
            if let Some(d) = &self.desc {
                write!(f, " \"{d}\"")?;
            }
            writeln!(f)?;
        }
        if let Some(d) = &self.default {
            spaces(f, depth + 1)?;
            writeln!(f, "default {d}")?;
        }

        if let Some(v) = &self.value {
            spaces(f, depth + 1)?;
            writeln!(f, "# current {v}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty_format(f, 1)
    }
}

#[derive(Debug)]
pub enum Entry {
    Variable(String),
    Menu(Menu),
}

#[derive(Debug)]
pub struct Menu {
    pub name: String,
    pub entries: Vec<Entry>,
}

impl Menu {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: vec![],
        }
    }
}

impl Menu {
    fn pretty_format(&self, f: &mut fmt::Formatter, kconfig: &KConfig, depth: i32) -> fmt::Result {
        if depth > 0 {
            spaces(f, depth - 1)?;
            writeln!(f, "menu \"{}\"", self.name)?;
        }
        for ent in &self.entries {
            match ent {
                Entry::Menu(m) => {
                    m.pretty_format(f, kconfig, depth + 1)?;
                }
                Entry::Variable(s) => {
                    let var = kconfig.vars.get(s);
                    if let Some(var) = var {
                        var.pretty_format(f, depth)?;
                    }
                }
            }
        }
        if depth > 0 {
            spaces(f, depth - 1)?;
            writeln!(f, "endmenu\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "menu \"{}\"", self.name)?;
        for ent in &self.entries {
            match ent {
                Entry::Menu(m) => {
                    m.fmt(f)?;
                }
                Entry::Variable(s) => {
                    writeln!(f, "  {}", s)?;
                }
            }
        }
        writeln!(f, "endmenu")?;
        Ok(())
    }
}

/// The structure to represent the contents of a Kconfig file
#[derive(Debug)]
pub struct KConfig {
    pub name: String,
    pub root: Menu,
    pub vars: IndexMap<String, Variable>,
}

impl KConfig {
    pub fn new() -> Self {
        Self {
            name: "config".to_string(),
            root: Menu::new("(top)"),
            vars: Default::default(),
        }
    }

    pub fn source(&mut self, other: Self) {
        self.vars.extend(other.vars);
    }

    pub fn add_var(&mut self, var: Variable) {
        self.vars.insert(var.name.to_string(), var);
        // self.
    }

    pub fn save(&self) -> IndexMap<String, Option<Value>> {
        self.vars
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    pub fn save_config(&self, dst: &str) -> io::Result<()> {
        let mut file = std::fs::File::create(dst)?;
        let settings = self.save();
        for (k, v) in &settings {
            match v {
                Some(v) => {
                    match v {
                        Value::Bool(false) => writeln!(&mut file, "# CONFIG_{k} is not set")?,
                        _ => writeln!(&mut file, "CONFIG_{k}={v}")?,
                    }
                    //
                }
                None => {
                    writeln!(&mut file, "# CONFIG_{k} is not set")?;
                    //
                }
            };
        }

        Ok(())
    }

    pub fn load_default(&mut self) {
        for (_k, v) in &mut self.vars {
            v.value = v.default.clone();
        }
    }

    pub fn load(&mut self, config_file: &str) -> io::Result<()> {
        let file = std::fs::File::open(config_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let res = parser::parse_config_line(&line?);
            if let Some((k, v)) = res {
                if let Some(var) = self.vars.get_mut(&k) {
                    var.value = Some(v);
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for KConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "mainmenu \"{}\"", self.name)?;
        f.write_str("\n")?;
        self.root.pretty_format(f, self, 0)?;
        Ok(())
    }
}
