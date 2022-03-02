#![feature(result_copied)] // TODO
pub use lighter_derive::lighter;

#[doc(hidden)]
pub mod __internal;

/*
TODO: tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/
