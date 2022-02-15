use proc_macro::TokenStream;
use proc_macro2::Span;
use std::{collections::BTreeMap, iter::Peekable, rc::Rc};
use syn::{
    parse_macro_input, spanned::Spanned, Expr, ExprLit, ExprMatch, Lit, LitStr, Pat, PatLit,
};

// TODO: error handling
// TODO: assert no attrs etc.
fn parse_arm(
    pat: Pat,
    body: Rc<Expr>,
    cases: &mut BTreeMap<Box<[u8]>, (Span, Rc<Expr>)>,
    wild: &mut Option<(Span, Rc<Expr>)>,
) {
    match pat {
        Pat::Lit(expr) => match *expr.expr {
            Expr::Lit(expr) => match expr.lit {
                Lit::Str(expr) => {
                    if let Some(_) =
                        cases.insert(expr.value().into_boxed_str().into(), (expr.span(), body))
                    {
                        panic!("duplicate keys");
                    }
                }
                _ => todo!("non-str lit"),
            },
            _ => todo!("non-lit expr"),
        },
        Pat::Or(expr) => {
            for expr in expr.cases {
                parse_arm(expr, body.clone(), cases, wild);
            }
        }
        Pat::Wild(expr) => assert!(wild.replace((expr.span(), body)).is_none()),
        x => todo!("non-lit pat {:?}", x),
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Match {
    Empty,
    Char(u8),
    Wild,
}

/*fn match_prefix(
    prefix: &mut Vec<u8>,
    cases: &mut BTreeMap<Box<[u8]>, (Span, Rc<Expr>)>,
    wild: &mut Option<(Span, Rc<Expr>)>,
) {
    println!("match_prefix {:?}", prefix);

    while let Some((key, (span, body))) =
        cases.next_if(|(key, _)| dbg!(key).starts_with(prefix.as_slice()))
    {
        match &key[prefix.len()..] {
            [] => {
                for _ in 0..prefix.len() {
                    print!("    ");
                }
                println!("body {:?}", body);

                //assert!(map.insert(Match::None, (span, body)).is_none());
            }
            [first, rest @ ..] => {
                for _ in 0..prefix.len() {
                    print!("    ");
                }
                println!("{} => match s[{}]:", *first, prefix.len());

                prefix.push(*first);
                let submatch = match_prefix(prefix, cases, wild);
                assert_eq!(prefix.pop(), Some(*first));

                // TODO: more specific Span for specific character
                //assert!(map.insert(Match::Some(first), (span, submatch)).is_none());
            }
        }
    }

    if let Some(body) = wild {
        for _ in 0..prefix.len() {
            print!(" ");
        }
        println!("_ => {:?}", body);

        //assert!(map.insert(Match::Wild, body.clone()).is_none());
    }

    //map
}*/

#[proc_macro]
pub fn lighter(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ExprMatch);

    let mut cases = BTreeMap::new();
    let mut wild = None;

    for arm in expr.arms {
        parse_arm(arm.pat, arm.body.into(), &mut cases, &mut wild);
    }

    //dbg!(cases, wild);
    for key in cases.keys() {
        println!("{:?}", key);
    }

    println!("match s.next() {{");
    // TODO: use Option?
    let mut last_key: Box<[u8]> = Box::new([]);
    for (key, (span, body)) in cases {
        let common_prefix = last_key.iter().zip(key.iter()).take_while(|(a, b)| a == b).count();

        if key.len() - common_prefix == 1 {
            println!("//opportunity for sharing");
        }
        
        // close match statements for bytes that
        // are different between last_key and key
        for _ in common_prefix..last_key.len() {
            println!("_ => panic!(\"default\"),");
            println!("}}");
        }

        // open new match statements for new prefix bytes
        for b in &key[common_prefix..] {
            println!("{} => match s.next() {{", b);
        }

        println!("None => panic!(\"found {:?}\"),", key);

        last_key = key;
    }
    for _ in last_key.iter() {
        println!("_ => panic!(\"default\"),");
        println!("}}");
    }
    println!("_ => panic!(\"default\"),");
    println!("}}");

    todo!()
}

/*
// TODO
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/
