# `lighter` - slightly better than a `match` 🔥

Say we have some Rust code that uses a `match` statement on a string:

```rust
pub fn greeting_id(greeting: &str) -> usize {
    match greeting {
        "hi" => 0,
        "hey" => 1,
        "hello" => 2,
        _ => 3,
    }
}
```

It turns out `match` statements with strings for patterns are [compiled](https://rust.godbolt.org/z/Tnz1oMja7) into a series of `if` statements, such that the above code is essentially equivalent to this:

```rust
pub fn greeting_id(greeting: &str) -> usize {
    if greeting == "hi" {
        0
    } else if greeting == "hey" {
        1
    } else if greeting == "hello" {
        2
    } else {
        3
    }
}
```

`rustc` and LLVM have [many tricks](https://llvm.org/pubs/2007-05-31-Switch-Lowering.pdf) to optimize match statements where the arguments are primitives such as [integers](https://rust.godbolt.org/z/vrTjevT1f) or [enums](https://rust.godbolt.org/z/j9v3Grcx5), but Rust doesn't seem to treat strings as "special" the way it does with numbers or enums in match statements. Instead, it falls back to calling the relevant [`PartialEq`](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html) implementation. This can be inefficient, especially if we have many strings and some of them share common prefixes. Here is `greeting_id` modified to use `lighter` instead of a plain `match`:

```rust
use lighter::lighter;

pub fn greeting_id(greeting: &str) -> usize {
    lighter! {
        match greeting {
            "hi" => 0,
            "hey" => 1,
            "hello" => 2,
            _ => 3,
        }
    }
}
```

During compilation, `lighter` essentially turns the flat match statement into a static [trie](https://en.wikipedia.org/wiki/Trie), where each `match` statement considers one character at a time:

```rust
pub fn greeting_id(greeting: &str) -> usize {
    let mut bytes = greeting.bytes();
    match bytes.next() {
        Some(b'h') => match bytes.next() {
            Some(b'i') => match bytes.next() {
                None => 0,
                _ => 3,
            },
            Some(b'e') => match bytes.next() {
                Some(b'y') => match bytes.next() {
                    None => 1,
                    _ => 3,
                },
                Some(b'l') => match bytes.next() {
                    Some(b'l') => match bytes.next() {
                        Some(b'o') => match bytes.next() {
                            None => 2,
                            _ => 3,
                        },
                        _ => 3,
                    },
                    _ => 3,
                },
                _ => 3,
            },
            _ => 3,
        },
        _ => 3,
    }
}
```

This may *look* somewhat gnarly compared to the original `match` without `lighter`, but by using byte literals (which are actually just `u8`s) we allow Rust and LLVM to use their full arsenal of optimizations for switches mapping numbers to numbers, resulting in [better code](https://rust.godbolt.org/z/zcxKhdWfd). The nested-`match` structure also means we only have to compare each character once: with a plain `match`, `greeting_id` compares its input against both the `h` in `"hi"` and the `h` in `"hello"`, whereas with `lighter`, `greeting_id` matches an `h` once and knows the suffixes it is looking for are either `i` or `ello`.

What's more, `lighter` doesn't just work with strings or slices: you can match on the output of any iterator of `u8`, or anything that can be turned into one. `lighter` can also match strings matching a prefix instead of or in addition to entire strings; see `lighter/examples/is_whitespace_2.rs`.
