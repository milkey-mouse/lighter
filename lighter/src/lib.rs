use once_cell::unsync::Lazy;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote, parse_str, spanned::Spanned, Arm, Expr, ExprLit, ExprMatch,
    Lit, LitByte, Pat, PatLit, PatTuple, PatTupleStruct, Path,
};

const QUOTE_SOME: Lazy<Path> = Lazy::new(|| parse_quote!(::core::option::Option::Some));
const QUOTE_NONE: Lazy<Path> = Lazy::new(|| parse_quote!(::core::option::Option::None));

fn insert_arm(m: &mut ExprMatch, case: &[u8], arm: Arm) {
    match case {
        // there is at least one byte left to match, let's find or create
        // another level of match statement for each next byte recursively
        [prefix, suffix @ ..] => {
            let found_m = m.arms.iter_mut().find_map(|m_arm| match &m_arm.pat {
                // try to find an existing arm for this prefix

                // this ugly match stmt is just to get at the
                // literal byte (e.g. 1 in Option::Some(b'\x01'))
                Pat::TupleStruct(PatTupleStruct {
                    path,
                    pat: PatTuple { elems, .. },
                    ..
                }) if elems.len() == 1 && *path == *QUOTE_SOME => match elems.first() {
                    Some(Pat::Lit(PatLit { expr, .. })) => match expr.as_ref() {
                        Expr::Lit(ExprLit {
                            lit: Lit::Byte(b), ..
                        }) if b.value() == *prefix => match m_arm.body.as_mut() {
                            Expr::Match(m_inner) => Some(m_inner),
                            a => panic!("non-match match arm {:?}", a),
                        },
                        _ => None,
                    },
                    a => panic!("weird arm {:?}", a),
                },
                _ => None,
            });

            let m_inner = match found_m {
                Some(m_inner) => m_inner,
                None => {
                    // arm does not yet exist for this prefix

                    // the format! is a workaround for a bug in
                    // LitByte::value where values created with
                    // LitByte::new are not parsed correctly
                    let mut b = parse_str::<LitByte>(&format!("b'\\x{:02x}'", prefix)).unwrap();
                    b.set_span(arm.pat.span());

                    // TODO: unhygienic! what if var is not i?
                    // match #m.expr causes inf loop for some reason
                    // TODO: parse_quote_spanned! ?
                    m.arms.push(parse_quote! {
                        ::core::option::Option::Some(#b) => match i.next() {},
                    });

                    match m.arms.last_mut().unwrap().body.as_mut() {
                        Expr::Match(m_inner) => m_inner,
                        _ => panic!(),
                    }
                }
            };

            insert_arm(m_inner, suffix, arm);
        }
        // we are at a leaf: for an n-character string, we're n matches deep,
        // and so we have no more chars to match. iff the iterator is empty &
        // thus the string we're matching is over, the original arm triggers
        [] => m.arms.push(Arm {
            pat: parse_quote!(::core::option::Option::None),
            ..arm
        }),
    }
}

// recursively append a wild/fallback case to every match expr
fn insert_wild(m: &mut ExprMatch, wild: Arm) {
    for arm in m.arms.iter_mut() {
        if let Expr::Match(arm_m) = arm.body.as_mut() {
            insert_wild(arm_m, wild.clone());
        }
    }

    m.arms.push(wild);
}

// TODO: error handling
// TODO: assert no attrs etc.
fn parse_arm(out: &mut ExprMatch, wild: &mut Option<Arm>, arm: Arm) {
    match arm.pat {
        Pat::Lit(ref pat_lit) => {
            //assert_eq!(pat_lit.attrs, []); // TODO: copy match attrs?
            match pat_lit.expr.as_ref() {
                Expr::Lit(expr_lit) => {
                    //assert_eq!(expr_lit.attrs, []);
                    match &expr_lit.lit {
                        Lit::Str(lit_str) => insert_arm(out, lit_str.value().as_bytes(), arm),
                        // TODO: handle if guards
                        _ => todo!("non-str lit"),
                    }
                }
                _ => todo!("non-lit expr"),
            }
        }
        Pat::Or(ref pat_or) => {
            assert_eq!(pat_or.attrs, []);
            for pat in &pat_or.cases {
                parse_arm(
                    out,
                    wild,
                    Arm {
                        pat: pat.clone(),
                        // TODO: clone each field separately
                        // (but keep span info)
                        ..arm.clone()
                    },
                )
            }
        }
        Pat::Wild(ref pat_wild) => {
            assert_eq!(pat_wild.attrs, []);
            assert!(wild.replace(arm).is_none());
        }
        x => todo!("non-lit pat {:?}", x),
    }
}

#[proc_macro]
pub fn lighter(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ExprMatch);
    //assert_eq!(expr.attrs, []);

    let mut wild = None;
    let mut out = ExprMatch {
        attrs: expr.attrs.clone(),
        match_token: expr.match_token,
        expr: expr.expr.clone(),
        brace_token: expr.brace_token,
        arms: Vec::new(),
    };

    for arm in expr.arms {
        parse_arm(&mut out, &mut wild, arm);
    }

    if let Some(w) = wild {
        insert_wild(&mut out, w);
    }

    out.to_token_stream().into()
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
