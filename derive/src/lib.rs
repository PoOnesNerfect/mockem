#![allow(
    clippy::blocks_in_if_conditions,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::manual_find,
    clippy::manual_let_else,
    clippy::manual_map,
    clippy::map_unwrap_or,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::range_plus_one,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::wrong_self_convention
)]

extern crate proc_macro;

mod parse;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn mock(_: TokenStream, token_stream: TokenStream) -> TokenStream {
    // token_stream.
    let input = parse_macro_input!(token_stream as parse::Item);

    input.to_token_stream().into()
}
