use core::{iter, mem};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, parse_str, spanned::Spanned, Arm, Expr,
    ExprArray, ExprMatch, Ident, Lit, LitByte, Pat, PatIdent,
};

// return the body of the arm of `m` with the given byte as its pattern, if it exists
fn find_arm(m: &mut ExprMatch, byte: u8) -> Option<&mut Expr> {
    for arm in m.arms.iter_mut() {
        // these ugly nested if statements are just to get at
        // the literal byte (e.g. the 1 in Some(Ok(b'\x01')))
        if let Pat::TupleStruct(expr) = &arm.pat {
            if expr.path == parse_quote!(::core::option::Option::Some) && expr.pat.elems.len() == 1
            {
                if let Some(Pat::TupleStruct(expr)) = expr.pat.elems.first() {
                    if expr.path == parse_quote!(::core::result::Result::Ok)
                        && expr.pat.elems.len() == 1
                    {
                        if let Some(Pat::Lit(expr)) = expr.pat.elems.first() {
                            if let Expr::Lit(expr) = expr.expr.as_ref() {
                                if let Lit::Byte(b) = &expr.lit {
                                    if b.value() == byte {
                                        return Some(&mut arm.body);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn insert_arm(expr: &mut Expr, case: &[u8], arm: Arm, match_prefix: bool) {
    match case {
        // we are at a leaf: for an n-character string, we're n matches deep,
        // and so we have no more chars to match. iff the iterator is empty &
        // thus the string we're matching is over, or if we are only matching
        // a prefix, the original arm's body runs
        [] => {
            let arm = Arm {
                pat: if match_prefix {
                    // when we are only matching a prefix, we don't care what
                    // comes after the prefix, or whether the string ends after
                    // the characters we've matched so far
                    parse_quote! {
                        ::core::option::Option::Some(::core::result::Result::Ok(_)) |
                        ::core::option::Option::None
                    }
                } else {
                    parse_quote!(::core::option::Option::None)
                },
                ..arm
            };

            match expr {
                // if expr is already a match statement, we can add to it as is
                Expr::Match(m) => m.arms.push(arm),

                // if our input is some other sort of statement, make it a wild
                // match arm that will come first and always execute, such that
                // arms added later (including the wild arm added to all match
                // statements by insert_wild) unreachable. this may seem silly,
                // but the goal is to trigger an "unreachable pattern" warning
                // when the user does something like the following:
                // ```
                // lighter! { match s {
                //     Prefix("") => println!("all strings start with the empty string"),
                //     "hi" => unreachable!(),
                //     _ => unreachable!(),
                // } }
                // ```
                expr => {
                    let e = mem::replace(expr, parse_quote!({}));
                    *expr = parse_quote! {
                        match __lighter_internal_iter.next() {
                            ::core::option::Option::Some(::core::result::Result::Err(e)) => ::core::result::Result::Err(e),
                            _ => #e,
                            #arm
                        }
                    };
                }
            }
        }

        // we are at a leaf for a prefix match: we don't need another match
        // statement a level after this to check iterator.next() = None, as
        // it's OK for the iterator to have more items after this one
        [prefix] if match_prefix => {
            // the format! is a workaround for a bug in
            // LitByte::value where values created with
            // LitByte::new are not parsed correctly
            let mut b = parse_str::<LitByte>(&format!("b'\\x{:02x}'", prefix)).unwrap();
            b.set_span(arm.pat.span());

            let arm = Arm {
                pat: parse_quote!(::core::option::Option::Some(::core::result::Result::Ok(#b))),
                ..arm
            };

            match expr {
                Expr::Match(m) => m.arms.push(arm),
                expr => {
                    let e = mem::replace(expr, parse_quote!({}));
                    *expr = parse_quote! {
                        match __lighter_internal_iter.next() {
                            ::core::option::Option::Some(::core::result::Result::Err(e)) => ::core::result::Result::Err(e),
                            _ => #e,
                            #arm
                        }
                    }
                }
            }
        }

        // there is at least one byte left to match, let's find or create
        // another level of match statement for each next byte recursively
        [prefix, suffix @ ..] => match expr {
            Expr::Match(m) => {
                let m_arm = match find_arm(m, *prefix) {
                    // an arm already exists with our prefix byte;
                    // insert our string's suffix relative to that
                    Some(m_arm) => m_arm,

                    // an arm does not yet exist for this prefix
                    None => {
                        // the format! is a workaround for a bug in
                        // LitByte::value where values created with
                        // LitByte::new are not parsed correctly
                        let mut b = parse_str::<LitByte>(&format!("b'\\x{:02x}'", prefix)).unwrap();
                        b.set_span(arm.pat.span());

                        // TODO: parse_quote_spanned! ?
                        m.arms.push(parse_quote! {
                            ::core::option::Option::Some(::core::result::Result::Ok(#b)) => match __lighter_internal_iter.next() {
                                ::core::option::Option::Some(::core::result::Result::Err(e)) => ::core::result::Result::Err(e),
                            },
                        });

                        m.arms.last_mut().unwrap().body.as_mut()
                    }
                };

                insert_arm(m_arm, suffix, arm, match_prefix);
            }
            expr => {
                // the format! is a workaround for a bug in
                // LitByte::value where values created with
                // LitByte::new are not parsed correctly
                // (TODO: report this bug)
                let mut b = parse_str::<LitByte>(&format!("b'\\x{:02x}'", prefix)).unwrap();
                b.set_span(arm.pat.span());

                // TODO: is there a simpler placeholder expression than {}?
                let e = mem::replace(expr, parse_quote!({}));
                *expr = parse_quote! {
                    match __lighter_internal_iter.next() {
                        _ => #e,
                        ::core::option::Option::Some(::core::result::Result::Ok(#b)) => match __lighter_internal_iter.next() {
                            ::core::option::Option::Some(::core::result::Result::Err(e)) => ::core::result::Result::Err(e),
                        },
                    }
                };
            }
        },
    }
}

// recursively append wild/fallback cases to every match expression that doesn't already have one
// TODO: what if wild case comes earlier than the last in the match statement?
fn insert_wild(expr: &mut Expr, wild: &[Arm], prefix: &mut Vec<u8>) {
    if let Expr::Match(m) = expr {
        let mut has_wild = false;
        for arm in m.arms.iter_mut() {
            match &arm.pat {
                Pat::Wild(_) => {
                    has_wild = true;
                    insert_wild(arm.body.as_mut(), wild, prefix);
                }
                Pat::TupleStruct(expr) => {
                    if expr.path == parse_quote!(::core::option::Option::Some)
                        && expr.pat.elems.len() == 1
                    {
                        if let Some(Pat::TupleStruct(expr)) = expr.pat.elems.first() {
                            if expr.path == parse_quote!(::core::result::Result::Ok)
                                && expr.pat.elems.len() == 1
                            {
                                if let Some(Pat::Lit(expr)) = expr.pat.elems.first() {
                                    if let Expr::Lit(expr) = expr.expr.as_ref() {
                                        if let Lit::Byte(b) = &expr.lit {
                                            prefix.push(b.value());
                                            insert_wild(arm.body.as_mut(), wild, prefix);
                                            assert_eq!(prefix.pop().unwrap(), b.value());
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    println!("non-Some(Ok) TupleStruct: {}", quote!(#expr).to_string());
                }
                p => panic!("weird pat when adding wild arms: {:?}", p),
            }
        }

        if !has_wild {
            for arm in wild {
                match &arm.pat {
                    Pat::Wild(_) => m.arms.push(arm.clone()),
                    Pat::Ident(PatIdent {
                        attrs,
                        by_ref,
                        mutability,
                        ident,
                        subpat,
                    }) => {
                        assert!(by_ref.is_none());
                        assert!(subpat.is_none());

                        // we need to handle two cases: one where we *did* read
                        // another byte (i.e. the iterator returned Some(b) and
                        // one where the iterator didn't read another byte (it
                        // returned None), but the wild case should still run
                        // with any previously read bytes (those in `prefix`)

                        // TODO: maybe this sort of wild arm should be added as
                        // we go along instead of after all the others, so that
                        // the following code would yield an "unreachable expr"
                        // error for the second match arm:
                        // ```
                        // match b {
                        //     s => println!("matched wild {}", s);
                        //     Prefix('x') => println!("matched x");
                        // }
                        // ```

                        // make a short name for arm.body because quote! would
                        // expand #arm.body as (#arm).body, not #(arm.body)
                        let body = &arm.body;

                        // add match arm for the Some(b) case
                        {
                            let len = prefix.len() + 1;
                            let bytes = prefix
                                .iter()
                                .map(|b| quote!(#b))
                                .chain(iter::once(quote!(__lighter_internal_last_byte)));

                            m.arms.push(Arm {
                                attrs: arm.attrs.clone(),
                                pat: parse_quote!(::core::option::Option::Some(::core::result::Result::Ok(
                                    __lighter_internal_last_byte
                                ))),
                                guard: arm.guard.clone(),
                                fat_arrow_token: arm.fat_arrow_token,
                                body: Box::new(parse_quote! {
                                    {
                                        let #mutability #ident: [u8; #len] = [#(#bytes),*];
                                        #body
                                    }
                                }),
                                comma: arm.comma,
                            });
                        }

                        // add match arm for the None case
                        {
                            let len = prefix.len();
                            let bytes = prefix.iter().map(|b| quote!(#b));

                            m.arms.push(Arm {
                                attrs: arm.attrs.clone(),
                                pat: parse_quote!(::core::option::Option::None),
                                guard: arm.guard.clone(),
                                fat_arrow_token: arm.fat_arrow_token,
                                body: Box::new(parse_quote! {
                                    {
                                        let #mutability #ident: [u8; #len] = [#(#bytes),*];
                                        #body
                                    }
                                }),
                                comma: arm.comma,
                            });
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

// TODO: error handling
// TODO: assert no attrs etc.
fn parse_arm(match_out: &mut Expr, wild: &mut Vec<Arm>, arm: Arm, prefix: bool) {
    match arm.pat {
        Pat::Lit(ref expr) => match expr.expr.as_ref() {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(expr) => insert_arm(match_out, expr.value().as_bytes(), arm, prefix),
                Lit::Byte(expr) => insert_arm(match_out, &[expr.value()], arm, prefix),
                Lit::Char(expr) => {
                    let mut buf = [0; 4];
                    let c = expr.value().encode_utf8(&mut buf).as_bytes();
                    insert_arm(match_out, c, arm, prefix)
                }
                // TODO: handle if guards
                _ => todo!("non-str lit"),
            },
            _ => todo!("non-lit expr"),
        },
        Pat::TupleStruct(expr)
            if expr.path == parse_quote!(Prefix) && expr.pat.elems.len() == 1 =>
        {
            let arm = Arm {
                pat: expr.pat.elems.into_iter().next().unwrap(),
                ..arm
            };

            parse_arm(match_out, wild, arm, true)
        }
        Pat::Or(expr) => {
            //for pat in &expr.cases {
            for pat in expr.cases {
                parse_arm(
                    match_out,
                    wild,
                    Arm {
                        attrs: arm.attrs.clone(),
                        pat,
                        guard: arm.guard.clone(),
                        fat_arrow_token: arm.fat_arrow_token,
                        body: arm.body.clone(),
                        comma: arm.comma,
                    },
                    prefix,
                )
            }
        }
        Pat::Ident(_) | Pat::Wild(_) => wild.push(arm),
        x => todo!("non-lit pat {:?}", x),
        //_ => todo!("non-lit pat"),
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

    let mut wild = Vec::new();
    // TODO: lighter! { match { Prefix("") => {} } } should consume 0 bytes/do nothing
    let mut match_out = Expr::Match(ExprMatch {
        attrs,
        match_token,
        expr: parse_quote_spanned!(expr.span()=> __lighter_internal_iter.next()),
        brace_token,
        arms: vec![parse_quote! {
            ::core::option::Option::Some(::core::result::Result::Err(e)) => ::core::result::Result::Err(e),
        }],
    });

    for arm in arms {
        parse_arm(&mut match_out, &mut wild, arm, false);
    }

    insert_wild(&mut match_out, &wild, &mut Vec::new());

    let krate = match crate_name("lighter") {
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        _ => parse_quote!(lighter),
    };

    // TODO
    let make_iter = quote_spanned! {expr.span()=>
        //(&mut &mut &mut ::#krate::__internal::Wrap(Some(#expr))).bytes()
        (&mut ::#krate::__internal::Wrap(Some(#expr))).bytes()
    };

    TokenStream::from(quote! {
        {
            use ::#krate::__internal::*;
            let mut __lighter_internal_iter = #make_iter;
            (&mut Wrap(#match_out)).maybe_unwrap()
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
