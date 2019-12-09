use std::io::{self, Write};

fn main() {
    loop {
        let mut line = String::new();
        print!("user> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();
        println!("{}", rep(line.as_str()));
    }
}

fn read(string: &str) -> &str {
    string
}

fn eval<'a>(ast: &'a str, env: &str) -> &'a str {
    ast
}

fn print(expression: &str) -> &str {
    expression
}

fn rep(string: &str) -> &str {
    print(eval(read(string), ""))
}