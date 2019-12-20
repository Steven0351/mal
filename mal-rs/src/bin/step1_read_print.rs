use std::io::{self, Write};
use mal_rs::reader::{Mal, Reader, LexerError};
use mal_rs::printer;

fn main() -> Result<(), LexerError> {
    loop {
        let mut line = String::new();
        print!("user> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();
        println!("{}", rep(line.as_str())?);
    }
}

fn read(string: &str) -> Result<Mal, LexerError> {
    Reader::read_str(string).read_form()
}

fn eval<'a>(ast: &'a Mal, env: &str) -> &'a Mal {
    ast
}

fn print(expression: &Mal) -> String {
    printer::print_str(expression)
}

fn rep(string: &str) -> Result<String, LexerError> {
    let mal = read(string)?;
    Ok(print(eval(&mal, "")))
}