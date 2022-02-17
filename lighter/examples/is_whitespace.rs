use lighter::lighter;
use std::io::{self, BufRead, Result};

fn main() -> Result<()> {
    println!("write a single character to stdin and I'll tell you if it's whitespace!");

    let line = io::stdin()
        .lock()
        .lines()
        .next()
        .expect("couldn't read line from stdin")?;

    println!(
        "{}",
        lighter! {
            match line {
                "\u{0009}" | "\u{000a}" | "\u{000b}" | "\u{000c}" |
                "\u{000d}" | "\u{0020}" | "\u{0085}" | "\u{00a0}" |
                "\u{1680}" | "\u{2000}" | "\u{2001}" | "\u{2002}" |
                "\u{2003}" | "\u{2004}" | "\u{2005}" | "\u{2006}" |
                "\u{2007}" | "\u{2008}" | "\u{2009}" | "\u{200a}" |
                "\u{2028}" | "\u{2029}" | "\u{202f}" | "\u{205f}" |
                "\u{3000}" => "whitespace",
                _ => "not whitespace",
            }
        }
    );

    Ok(())
}
