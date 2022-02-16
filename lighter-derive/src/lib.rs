use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, parse_str, spanned::Spanned, Arm, Expr,
    ExprLit, ExprMatch, Ident, Lit, LitByte, Pat, PatLit, PatTuple, PatTupleStruct,
};

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
                }) if elems.len() == 1 && *path == parse_quote!(::core::option::Option::Some) => match elems.first() {
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

                    // TODO: parse_quote_spanned! ?
                    m.arms.push(parse_quote! {
                        ::core::option::Option::Some(#b) => match __lighter_internal_iter.next() {},
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
fn parse_arm(match_out: &mut ExprMatch, wild: &mut Option<Arm>, arm: Arm) {
    match &arm.pat {
        Pat::Lit(pat_lit) => match pat_lit.expr.as_ref() {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Str(lit_str) => insert_arm(match_out, lit_str.value().as_bytes(), arm),
                // TODO: handle if guards
                _ => todo!("non-str lit"),
            },
            _ => todo!("non-lit expr"),
        },
        Pat::Or(pat_or) => {
            for pat in &pat_or.cases {
                parse_arm(
                    match_out,
                    wild,
                    Arm {
                        attrs: arm.attrs.clone(),
                        pat: pat.clone(),
                        guard: arm.guard.clone(),
                        fat_arrow_token: arm.fat_arrow_token,
                        body: arm.body.clone(),
                        comma: arm.comma,
                    },
                )
            }
        }
        Pat::Wild(_) => assert!(wild.replace(arm).is_none()),
        //x => todo!("non-lit pat {:?}", x),
        _ => todo!("non-lit pat"),
    }
}

#[proc_macro]
pub fn lighter(input: TokenStream) -> TokenStream {
    let ExprMatch {
        attrs,
        match_token,
        expr,
        brace_token,
        arms,
    } = parse_macro_input!(input as ExprMatch);
    if !attrs.is_empty() {
        panic!("I don't know what to do with attributes on a match statement");
    }

    let mut wild = None;
    let mut match_out = ExprMatch {
        attrs,
        match_token,
        expr: parse_quote_spanned!(expr.span()=> __lighter_internal_iter.next()),
        brace_token,
        arms: vec![], // TODO
    };

    for arm in arms {
        parse_arm(&mut match_out, &mut wild, arm);
    }

    if let Some(w) = wild {
        insert_wild(&mut match_out, w);
    }

    let krate = match crate_name("lighter") {
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        _ => parse_quote!(lighter),
    };

    let make_iter = quote_spanned! {expr.span()=>
        (&mut &mut &mut ::#krate::__internal::Wrap(Some(#expr))).bytes()
    };

    TokenStream::from(quote! {
        {
            use ::#krate::__internal::*;
            let mut __lighter_internal_iter = #make_iter;
            #match_out
        }
    })
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
