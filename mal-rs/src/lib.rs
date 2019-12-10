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

        pub fn read_form(&mut self) -> Result<Mal, LexerError> {
            let token = self.next().ok_or(LexerError::UnsupportedSyntax)?;
            let open = String::from("(");

            if token.eq(&open) {
                self.read_list()
            } else {
                self.read_atom()
            }
        }

        fn read_list(&mut self) -> Result<Mal, LexerError> {
            let close = String::from(")");
            let mut list: Vec<Mal> = vec![];

            while let Some(raw_token) = self.next() {
                if raw_token.eq(&close) {
                    break;
                } else {
                    list.push(self.read_form()?);
                }
            }
            
            Ok(Mal::List(list))
        }

        fn read_atom(&self) -> Result<Mal, LexerError> {
            let raw_token = self.peek();

            match raw_token {
                Some(token) if token.eq("true") => Ok(Mal::True),
                Some(token) if token.eq("false") => Ok(Mal::False),
                Some(token) if token.eq("nil") => Ok(Mal::Nil),
                Some(token) if STRING_REGEX.is_match(token) => lex_string(token),
                Some(token) if INT_REGEX.is_match(token) => lex_int(token),
                Some(token) if token.len() == 1 && SYMBOL_REGEX.is_match(token) => lex_symbol(token),
                _ => Err(LexerError::UnsupportedSyntax),
            }
        }
    }

    lazy_static! {
        static ref ALL_TOKENS: Regex = Regex::new(r#"(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();
        static ref STRING_REGEX: Regex = Regex::new(r#""(?:\\.|[^\\"])*"?"#).unwrap();
        static ref QUOTE_REGEX: Regex = Regex::new(r#"""#).unwrap();
        static ref INT_REGEX: Regex = Regex::new(r#"[\d*]"#).unwrap();
        static ref SYMBOL_REGEX: Regex = Regex::new(r"[+-\\*/]").unwrap();
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
        if string.ends_with("\"") {
            let mal_string = &string.as_str()[1..string.len() - 1];
            let mal_string = Mal::Str(String::from(mal_string));
            Ok(mal_string)
        } else {
            Err(LexerError::UnbalancedString)
        }
    }

    fn lex_symbol(string: &String) -> Result<Mal, LexerError> {
        Ok(Mal::Symbol(string.to_string()))
    }

    fn lex_int(string: &String) -> Result<Mal, LexerError> {
        if INT_REGEX.captures_iter(string).count() == string.len() {
            string.parse()
                .map_err(|_| LexerError::NaN)
                .map(|i| Mal::Int(i))
        } else {
            Err(LexerError::NaN)
        }
    }

    #[derive(Debug, Fail)]
    pub enum LexerError {
        #[fail(display = "Unexpected EOF")]
        UnbalancedString,
        #[fail(display = "Not a Number")]
        NaN,
        #[fail(display = "Syntax Error")]
        UnsupportedSyntax,
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
        fn reader_returns_mal_int() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("123");
            let mal_int = reader.read_form()?;
            if let Mal::Int(i) = mal_int {
                assert_eq!(123, i);
            } else { 
                panic!("Did not generate correct AST");
            }

            let mut reader = Reader::read_str("123  ");
            let mal_int = reader.read_form()?;
            if let Mal::Int(i) = mal_int {
                assert_eq!(123, i);
                Ok(())
            } else { 
                panic!("Did not generate correct AST");
            }
        }

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
        fn symbol_regex_captures_symbols() {
            let symbol_string = "+ - * /";
            assert_eq!(SYMBOL_REGEX.captures_iter(symbol_string).count(), 4);
        }

        #[test]
        fn lex_string_strips_quotes() -> Result<(), LexerError> {
            let string = String::from("\"This is a string\"");
            let string = lex_string(&string)?;
            
            if let Mal::Str(string) = string {
                assert!(string.eq("This is a string"));
            } else {
                panic!("No match");
            }

            Ok(())
        }

        #[test]
        fn lex_string_keeps_inner_escaped_quotes() -> Result<(), LexerError> {
            let string = String::from("\"This is a \\\"fancy\\\" string\"");
            let string = lex_string(&string)?;

            if let Mal::Str(string) = string {
                assert!(string.eq(r#"This is a \"fancy\" string"#));
            } else {
                panic!("No Match");
            }

            Ok(())
        }

        #[test]
        fn lex_int_parses_succesfully() -> Result<(), LexerError> {
            let int = String::from("312");
            let int = lex_int(&int)?;

            if let Mal::Int(int) = int {
                assert_eq!(int, 312);
            } else {
                panic!("Did not properly parse int of value 312");
            }

            Ok(())
        }
    }
}