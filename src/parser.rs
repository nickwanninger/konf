
use super::*;

use pest::{
    iterators::{Pairs},
    Parser,
};


macro_rules! unexpected_parser_syntax {
    ($pair:expr) => {
        unimplemented!(
            "unexpected parser rule: {:#?}\n\n {:#?}",
            $pair.as_rule(),
            $pair
        )
    };
}


#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
struct KonfParser;


impl Config {
    fn parse_fields(&mut self, pairs: Pairs<Rule>) {
    }
}


impl KConfig {
    fn parse_top_level(&mut self, pairs: Pairs<Rule>) {
        for pair in pairs {
            match pair.as_rule() {
                Rule::mainmenu => {
                    let inner = pair.clone().into_inner().next().unwrap();
                    let value = inner.as_span().as_str().replace("\"", "");
                    self.name = value;
                },

                Rule::config => {
                    println!("config: {}", pair);
                    let inner = pair.clone().into_inner().next().unwrap();
                    let name = inner.as_span().as_str().to_string();
                    let mut config = Config {
                        name,
                        ty: None
                    };
                    config.parse_fields(pair.into_inner());
                    self.entries.push(Entry::Config(config));
                },
                _ => unexpected_parser_syntax!(pair),
            }
        }
    }
}

fn kconfig_from_pairs(pairs: Pairs<Rule>) -> KConfig {
    for pair in pairs {
        match pair.as_rule() {
            Rule::kconfig => {
                let mut kconfig = KConfig {
                    name: "configuration".to_string(),
                    entries: vec![],
                };

                kconfig.parse_top_level(pair.into_inner());

                return kconfig;
            }
            _ => unexpected_parser_syntax!(pair),
        };
    }
    todo!();
}

pub fn parse(file: &str) -> Result<KConfig> {
    match KonfParser::parse(Rule::kconfig, file) {
        Ok(pairs) => {
            println!("pairs: {}", pairs);
            let kconfig = kconfig_from_pairs(pairs);
            println!("kconfig: {:?}", kconfig);
            Ok(kconfig)
        }
        Err(mut error) => {
            error = error.renamed_rules(|rule| format!("{:?}", rule));
            println!("Mapping parse error\n{}", error);
            Err(Error)
        }
    }
}
