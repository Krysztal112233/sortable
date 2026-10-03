//! Proc-macros for `sortable`, re-exported as `sortable::NamedRow` when the
//! main crate's `derive` feature is enabled.

use proc_macro::TokenStream;

/// Derives `Row` and `NamedRow` for a struct with named fields, making it
/// usable as the row type of a `sortable::Table`.
///
/// Not implemented yet: this crate currently only scaffolds the workspace
/// split.
#[proc_macro_derive(NamedRow)]
pub fn derive_named_row(_input: TokenStream) -> TokenStream {
    quote::quote! {
        ::core::compile_error!("NamedRow derive is not implemented yet");
    }
    .into()
}
