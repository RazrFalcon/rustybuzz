//! `nom` combinator for ragel-style scan.
//! 
//! Takes several combinators and matches the longest one. If two are equally long, take the
//! one that comes first sequentially.

use nom::{Parser, IResult, InputLength};
use nom::error::ParseError;

pub(crate) trait Scan<I, O, E> {
    fn run_scan(&mut self, input: I) -> IResult<I, O, E>;
}

pub(crate) fn scan<
    I: Clone, O, E: ParseError<I>,
    List: Scan<I, O, E>,
>(mut list: List) -> impl FnMut(I) -> IResult<I, O, E> {
    move |i| list.run_scan(i)
}

macro_rules! impl_scan {
    ([$($gen_name: ident),*] $first: ident $(,)? $($rest: ident),*) => {
        impl_scan_inner!($($gen_name),*);
        impl_scan!([$($gen_name,)* $first] $($rest),*);
    };
    ([$($gen_name: ident),*]) => {
        impl_scan_inner!($($gen_name),*);
    };
}

macro_rules! impl_scan_inner {
    ($first:ident, $($gen_name:ident),*) => {
        #[allow(non_snake_case)]
        impl <
            In: InputLength + Clone, Out, Err: ParseError<In>,
            $first: Parser<In, Out, Err>,
            $($gen_name: Parser<In, Out, Err>),*
        > Scan<In, Out, Err> for ($first, $($gen_name),*) {
            fn run_scan(&mut self, input: In) -> IResult<In, Out, Err> {
                let (ref mut $first, $(ref mut $gen_name),*) = self;

                // The original result. 
                let mut current_result = match $first.parse(input.clone()) {
                    // Recoverable error.
                    Err(nom::Err::Error(e)) => Err(e),
                    // Irrrecoverable error.
                    Err(e) => return Err(e),
                    // Success.
                    Ok((i, o)) => Ok((i, o))
                };

                // Iterate over the parsers.
                $({
                    // Run the parser and match results.
                    match (current_result, $gen_name.parse(input.clone())) {
                        // If they are both errors, "or" them together.
                        (Err(e1), Err(nom::Err::Error(e2))) => {
                            current_result = Err(e1.or(e2))
                        },
                        // If the second is an error, just ignore it.
                        (Ok((i, o)), Err(nom::Err::Error(_))) => {
                            current_result = Ok((i, o))
                        },
                        // Bubble up irrecoverable errors.
                        (_, Err(e)) => return Err(e),
                        // If the first is an error, but the second is not, return the second.
                        (Err(_), Ok((i, o))) => {
                            current_result = Ok((i, o))
                        },
                        // If they are both Ok, compare the lengths.
                        (Ok((i1, o1)), Ok((i2, o2))) => {
                            if i2.input_len() < i1.input_len() {
                                current_result = Ok((i2, o2))
                            } else {
                                current_result = Ok((i1, o1))
                            }
                        }
                    }
                })*

                current_result.map_err(nom::Err::Error)
            }
        }
    };
}

impl_scan!([A, B] C, D, E, F, G, H, I, J, K, L, M, N, O, P);
