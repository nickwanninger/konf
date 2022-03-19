use super::*;
use logos::{Lexer, Logos};
use std::ffi::OsStr;
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

    #[token("config")]
    Config,

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

struct Tokenizer<'a> {
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

impl<'a> Tokenizer<'a> {
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
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> std::result::Result<KConfig, &'static str> {
    let file_text = std::fs::read_to_string(path.as_ref());
    if let Err(e) = file_text {
        panic!("Failed to read: {}", e);
    }
    let mut toks = Tokenizer::new(file_text.as_ref().unwrap());

    let mut kconfig = KConfig {
        name: "config".to_string(),
        entries: vec![],
    };

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
                    Some(Token::String(name)) => kconfig.name = name.to_string(),
                    _ => return Err("Invalid option to `mainmenu`"),
                };
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

                            break;
                        }

                        kconfig.entries.push(Entry::Variable(var));
                    }
                    _ => return Err("Invalid name for `config`"),
                };
            }

            // "source" STRING
            Token::Source => {
                if let Some(s) = toks.accept_string() {
                    // get the parent path of the current kconfig
                    let target = path
                        .as_ref()
                        .canonicalize()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join(s);
                    dbg!(&target);

                    kconfig.source(parse_file(target)?);
                    // s is a path, relative to where?
                } else {
                    return Err("invalid argument to `source`");
                }
            }
            _ => return Err("invalid top level token"),
        }
    }

    Ok(kconfig)
}
