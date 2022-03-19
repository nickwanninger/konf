use super::*;
use logos::{Lexer, Logos};
use std::path::Path;

fn string_tokenize<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<&'a str> {
    let slice = lex.slice();
    Some(&slice[1..slice.len() - 1])
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
enum Token<'a> {
    #[token("mainmenu")]
    MainMenu,

    #[token("source")]
    Source,

    #[token("menu")]
    Menu,
    #[token("endmenu")]
    EndMenu,

    #[token("config")]
    Config,

    #[token("default")]
    Default,

    #[token("y")]
    Yes,

    #[token("n")]
    No,

    #[token("=")]
    Equals,

    #[regex("[A-Z_]+")]
    Name(&'a str),

    #[regex("\"([[^\"].]+)\"", string_tokenize)]
    String(&'a str),

    #[regex("(bool|int|string)", |lex| super::Type::new(lex.slice()))]
    Type(Type),

    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

struct Parser<'a> {
    toks: std::iter::Peekable<Lexer<'a, Token<'a>>>,
}

macro_rules! accept {
    ($method:ident, $variant:ident, $t:ty) => {
        fn $method(&mut self) -> Option<$t> {
            if let Some(&Token::$variant(x)) = self.toks.peek() {
                self.toks.next();
                Some(x)
            } else {
                None
            }
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            toks: Token::lexer(text).peekable(),
        }
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        self.toks.next()
    }

    pub fn peek(&mut self) -> Option<Token<'a>> {
        self.toks.peek().copied()
    }

    accept!(accept_string, String, &'a str);
    accept!(accept_type, Type, Type);

    pub fn parse_value(&mut self) -> Option<Value> {
        let val = self.peek();
        if let Some(val) = val {
            match val {
                Token::Yes => {
                    self.next();
                    return Some(Value::Bool(true));
                }
                Token::No => {
                    self.next();
                    return Some(Value::Bool(false));
                }
                _ => return None,
            }
        }
        return None;
    }
}

impl Menu {
    fn parse<'a>(
        &mut self,
        path: &Path,
        toks: &mut Parser<'a>,
        vars: &mut IndexMap<String, Variable>,
    ) -> std::result::Result<(), &'static str> {
        while let Some(tok) = toks.next() {
            // top level options:
            //    MainMenu
            //    Config
            // TODO:
            //    Include
            //
            match tok {
                // "mainmenu"
                Token::MainMenu => {
                    let name = toks.next();
                    match name {
                        Some(Token::String(name)) => self.name = name.to_string(),
                        _ => return Err("Invalid option to `mainmenu`"),
                    };
                }

                Token::Menu => {
                    if let Some(s) = toks.accept_string() {
                        let mut m = Menu::new(s);
                        m.parse(path, toks, vars)?;
                        self.entries.push(Entry::Menu(m));
                    }
                }
                Token::EndMenu => {
                    // consume the endmenu
                    break;
                }

                // "config" NAME
                Token::Config => {
                    // get the NAME
                    match toks.next() {
                        Some(Token::Name(name)) => {
                            let mut var = Variable::new(name);
                            loop {
                                // capture the type of the variable
                                if let Some(t) = toks.accept_type() {
                                    var.ty = Some(t);
                                    // Capture the optional description after the type
                                    if let Some(s) = toks.accept_string() {
                                        var.desc = Some(s.to_string());
                                    }
                                    continue;
                                }

                                if let Some(Token::Default) = toks.peek() {
                                    toks.next();

                                    if let Some(val) = toks.parse_value() {
                                        var.default = Some(val);
                                    } else {
                                        return Err("Missing argument for `default`");
                                    }
                                }

                                break;
                            }

                            vars.insert(var.name.clone(), var);
                            self.entries.push(Entry::Variable(name.to_string()));
                        }
                        _ => return Err("Invalid name for `config`"),
                    };
                }

                // "source" STRING
                Token::Source => {
                    if let Some(s) = toks.accept_string() {
                        // get the parent path of the current kconfig
                        let target = path.canonicalize().unwrap().parent().unwrap().join(s);
                        let other = parse_file(target)?;
                        // TOAD: merge the menu bro
                        vars.extend(other.vars);
                    } else {
                        return Err("invalid argument to `source`");
                    }
                }
                _ => return Err("invalid top level token"),
            }
        }
        Ok(())
    }
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> std::result::Result<KConfig, &'static str> {
    let file_text = std::fs::read_to_string(path.as_ref());
    if let Err(e) = file_text {
        panic!("Failed to read: {}", e);
    }
    let mut toks = Parser::new(file_text.as_ref().unwrap());

    let mut kconfig = KConfig::new();

    kconfig
        .root
        .parse(path.as_ref(), &mut toks, &mut kconfig.vars)?;

    kconfig.name = kconfig.root.name.clone();

    Ok(kconfig)
}

/// Return a variable/value mapping, parsed from a line of `.config`. There are a few
/// cases that this function can handle:
///
///
pub fn parse_config_line(line: &str) -> Option<(String, Value)> {
    // First, handle "is not set". If this regex matches, it really just means CONFIG_X=n.
    let unset_match = Regex::new(r"# CONFIG_([^ ]+) is not set").unwrap();
    if let Some(caps) = unset_match.captures(line) {
        return Some((caps[1].to_string(), Value::Bool(false)));
    }

    // Create a parser for the line
    let mut toks = Parser::new(line);
    // Try to parse a Name
    if let Token::Name(s) = toks.next()? {
        // then an Equals
        if let Token::Equals = toks.next()? {
            // Then a value
            let v = toks.parse_value()?;
            // And return it with the `CONFIG_` stripped from the front
            return Some((s.strip_prefix("CONFIG_").unwrap().to_string(), v));
        }
    }
    None
}
