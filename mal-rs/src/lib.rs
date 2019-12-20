#![feature(pattern)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate failure_derive;

pub mod printer {
    use super::reader::{self, Reader};

    pub fn print_str(mal: &reader::Mal) -> String {
        format!("{}", mal)
    }
}

pub mod reader {
    use regex::Regex;
    use std::fmt;
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
            let token = self.tokens.get(self.position);
            self.position += 1;
            token
        }

        pub fn peek(&self) -> Option<&String> {
            self.tokens.get(self.position)
        }

        pub fn read_form(&mut self) -> Result<Mal, LexerError> {
            let open = String::from("(");
            let mut mal = Mal::Nil;

            if let Some(token) = self.peek() {
                if token.eq(&open) {
                    mal = self.read_list()?;
                } else {
                    mal = self.read_atom()?;
                }
            }

            Ok(mal)
        }

        fn read_list(&mut self) -> Result<Mal, LexerError> {
            let mut list: Vec<Mal> = vec![];
 
            while let Some(_) = self.next() {
                let mal = self.read_form()?;
                list.push(mal);

                if let Mal::Nil = list.last().unwrap() {
                    break;
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
                Some(token) if token.parse::<i32>().is_ok() => lex_int(token),
                Some(token) => lex_symbol(token),
                _ =>  Ok(Mal::Nil),
            }
        }
    }

    lazy_static! {
        static ref ALL_TOKENS: Regex = Regex::new(r#"(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();
        static ref STRING_REGEX: Regex = Regex::new(r#""(?:\\.|[^\\"])*"?"#).unwrap();
        static ref QUOTE_REGEX: Regex = Regex::new(r#"""#).unwrap();
        static ref INT_REGEX: Regex = Regex::new(r#"[-?(=\d)]"#).unwrap();
        //static ref SYMBOL_REGEX: Regex = Regex::new(r"[+-\\*/]").unwrap();
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
        string.parse()
            .map_err(|_| LexerError::NaN)
            .map(|i| Mal::Int(i))
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
        Int(i32),
        Str(String),
        Nil,
        True,
        False,
        Symbol(String),
        List(Vec<Mal>),
    }

    impl fmt::Display for Mal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let string = match self {
                Mal::Int(i) => format!("{}", i),
                Mal::Str(s) => format!("\"{}\"", s),
                Mal::Nil => String::from("nil"),
                Mal::True => String::from("true"),
                Mal::False => String::from("false"),
                Mal::Symbol(s) => format!("{}", s),
                Mal::List(vec) => {
                    let mut string = String::from("(");
                    for item in vec {
                        if let Mal::Nil = item { continue; }
                        string.push_str(format!("{} ", item).as_str());
                    }
                    string.pop();
                    string.push_str(")");
                    string
                }
            };

            write!(f, "{}", string)
        }
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
        fn reader_returns_mal_true() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("true");
            let mal_true = reader.read_form()?;
            if let Mal::True = mal_true {
                Ok(())
            } else {
                panic!("true isn't true?");
            }
        }

        #[test]
        fn reader_returns_mal_false() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("false");
            let mal_false = reader.read_form()?;
            if let Mal::False = mal_false {
                Ok(())
            } else {
                panic!("false isn't false?");
            }
        }

        #[test]
        fn reader_returns_mal_nil() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("nil");
            let mal_nil = reader.read_form()?;
            if let Mal::Nil = mal_nil {
                Ok(())
            } else {
                panic!("nil isn't nil");
            }
        }

        #[test]
        fn reader_returns_mal_symbol() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("+");
            let mal_symbol = reader.read_form()?;
            if let Mal::Symbol(symbol) = mal_symbol {
                assert!(symbol.eq("+"));
                Ok(())
            } else {
                panic!("Could not parse symbol from +")
            }
        }

        #[test]
        fn reader_returns_mal_string() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("\"This is a string\"");
            let mal_string = reader.read_form()?;
            if let Mal::Str(string) = mal_string {
                assert!(string.eq("This is a string"));
                Ok(())
            } else {
                panic!("Could not parse string from \"This is a string\"");
            }
        }

        #[test]
        fn reader_returns_simple_mal_list() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("( 123 456 789 )");
            let mal_list = reader.read_form()?;
            if let Mal::List(list) = mal_list {
                let mut vec: Vec<i32> = vec![];
                for elem in &list {
                    if let Mal::Int(i) = elem {
                        vec.push(*i);
                    }
                }
                assert_eq!(vec[0], 123);
                assert_eq!(vec[1], 456);
                assert_eq!(vec[2], 789);
                
                if let Mal::Nil = &list[3] {
                    Ok(())
                } else {
                    panic!("List not nil terminated");
                }
            } else {
                panic!("Could not parse list from (123 123 123)");
            }
        }

        #[test]
        fn reader_returns_nested_mal_list() -> Result<(), LexerError> {
            let mut reader = Reader::read_str("( + 2 (* 3 4) )");
            let mal_list = reader.read_form()?;
            if let Mal::List(list) = mal_list {
                assert_eq!(list.len(), 4);

                if let Mal::Symbol(sym) = &list[0] {
                    assert!(sym.eq("+"));
                } else {
                    panic!("Element 0 was not +");
                }

                if let Mal::Int(i) = &list[1] {
                    assert_eq!(2, *i);
                } else {
                    panic!("Element 1 was not 2");
                }

                if let Mal::List(list) = &list[2] {
                    assert_eq!(list.len(), 4);

                    if let Mal::Symbol(sym) = &list[0] {
                        assert!(sym.eq("*"));
                    } else {
                        panic!("Inner List Element 0 was not *");
                    }

                    if let Mal::Int(i) = &list[1] {
                        assert_eq!(3, *i);
                    } else {
                        panic!("Inner List Element 1 was not 3");
                    }

                    if let Mal::Int(i) = &list[2] {
                        assert_eq!(4, *i);
                    } else {
                        panic!("Inner List Element 2 was not 4")
                    }

                    if let Mal::Nil = &list[3] {
                        
                    } else {
                        panic!("Inner list was not nil terminated");
                    }

                } else {
                    panic!("Element 2 was not (* 3 4)");
                }

                if let Mal::Nil = &list[3] {
                    Ok(())
                } else {
                    panic!("List was not nil terminated");
                }
            } else {
                panic!("Could not parse list from ( + 2 (* 3 4) )");
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