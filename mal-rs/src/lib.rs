#![feature(pattern)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate failure_derive;

pub mod reader {
    use regex::Regex;
    use std::str::pattern::Pattern;

    #[derive(Debug)]
    pub struct Reader {
        tokens: Vec<String>,
        position: usize,
    }

    impl Reader {
        pub fn read_str(string: &str) -> Reader {
            Reader {
                tokens: tokenize(string),
                position: 0
            }
        }

        pub fn next(&mut self) -> Option<&String> {
            // On the first read, we do not want to increment the position because we will 
            // always end up starting on the second token instead of the first.
            if self.position != 0 {
                self.position += 1;
            }

            let token = self.tokens.get(self.position);
            token
        }

        pub fn peek(&self) -> Option<&String> {
            self.tokens.get(self.position)
        }

        pub fn read_form(&mut self) -> Option<Mal> {
            let token = self.next()?;
            let open = String::from("(");

            if token.eq(&open) {
                self.read_list()
            } else {
                self.read_atom()
            }
        }

        fn read_list(&mut self) -> Option<Mal> {
            let close = String::from(")");
            let mut list: Vec<Mal> = vec![];

            while let Some(raw_token) = self.next() {
                if raw_token.eq(&close) {
                    break;
                } else {
                    list.push(self.read_form()?);
                }
            }
            
            Some(Mal::List(list))
        }

        fn read_atom(&self) -> Option<Mal> {
            let raw_token = self.peek();

            match raw_token {
                Some(token) if token.is_contained_in("0123456789") => Some(Mal::Int(token.parse().unwrap())),
                Some(token) if STRING_REGEX.is_match(token) => lex_string(token).ok(),
                _ => None,
            }
        }
    }

    lazy_static! {
        static ref ALL_TOKENS: Regex = Regex::new(r#"(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();
        static ref STRING_REGEX: Regex = Regex::new(r#""(?:\\.|[^\\"])*"?"#).unwrap();
        static ref QUOTE_REGEX: Regex = Regex::new(r#"""#).unwrap();
    }

    fn tokenize(string: &str) -> Vec<String> {
        ALL_TOKENS.find_iter(string)
            .filter_map(|s|
                if s.as_str().is_empty() {
                    None
                } else {
                    Some(s.as_str().to_string())
                }
            )
            .collect()
    }

    fn lex_string(string: &String) -> Result<Mal, LexerError> {
        if QUOTE_REGEX.captures_iter(string).count() == 2 {
            let mal_string = Mal::Str(QUOTE_REGEX.replace_all(string.as_str(), "").to_string());
            Ok(mal_string)
        } else {
            Err(LexerError::UnbalancedString)
        }
    }

    #[derive(Debug, Fail)]
    enum LexerError {
        #[fail(display = "Unexpected EOF")]
        UnbalancedString,
    }

    pub enum Mal {
        Int(u32),
        Str(String),
        Nil,
        True,
        False,
        Symbol(String),
        List(Vec<Mal>),
    } 
    
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn captures_single_special_characters() {
            let vec = tokenize("    (  ) ~@[]{}'`~^    ,   @");
            assert_eq!(vec, ["(", ")", "~@", "[", "]", "{", "}", "'", "`", "~", "^", "@"]);
        }

        #[test]
        fn captures_string() {
            let vec = tokenize("\"This is a string\"");
            assert_eq!(vec, ["\"This is a string\""]);
        }

        #[test]
        fn string_regex_finds_string() {
            let string = "\"This is a string\"";
            assert!(STRING_REGEX.is_match(string));
        }

        #[test]
        fn string_regex_finds_unbalanced_string() {
            let string = "\"This is a string";
            assert!(STRING_REGEX.is_match(string));
        }

        #[test]
        fn quote_regex_finds_string_quotes() {
            assert_eq!(QUOTE_REGEX.captures_iter("\"This is a string\"").count(), 2);
        }
    }
}