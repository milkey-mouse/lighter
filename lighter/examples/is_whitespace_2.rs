use lighter::lighter;
use std::io::{self, BufRead, Result};

fn main() -> Result<()> {
    println!("Write a line to stdin and I'll tell you if it's all whitespace!");

    let line = io::stdin()
        .lock()
        .lines()
        .next()
        .expect("couldn't read line from stdin")?;

    let mut bytes = line.bytes().peekable();

    while bytes.peek().is_some() {
        lighter! {
            match &mut bytes {
                Prefix("\u{0009}") | Prefix("\u{000a}") | Prefix("\u{000b}") | Prefix("\u{000c}") |
                Prefix("\u{000d}") | Prefix("\u{0020}") | Prefix("\u{0085}") | Prefix("\u{00a0}") |
                Prefix("\u{1680}") | Prefix("\u{2000}") | Prefix("\u{2001}") | Prefix("\u{2002}") |
                Prefix("\u{2003}") | Prefix("\u{2004}") | Prefix("\u{2005}") | Prefix("\u{2006}") |
                Prefix("\u{2007}") | Prefix("\u{2008}") | Prefix("\u{2009}") | Prefix("\u{200a}") |
                Prefix("\u{2028}") | Prefix("\u{2029}") | Prefix("\u{202f}") | Prefix("\u{205f}") |
                Prefix("\u{3000}") => {},
                _ => {
                    println!("not whitespace");
                    return Ok(());
                },
            }
        }
    }

    println!("whitespace");
    Ok(())
}
