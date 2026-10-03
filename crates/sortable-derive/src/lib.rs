//! Proc-macros for `sortable`, re-exported as `sortable::NamedRow` when the
//! main crate's `derive` feature is enabled.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives `NamedRow` for a struct with named fields, making it usable as
/// the row type of a `sortable::Table`.
///
/// Everything a table needs is generated from the fields: the column names
/// (`NamedRow::NAMES`, the headers) from the field names, the column kinds
/// (`NamedRow::KINDS`) from the field types via `ColumnType`, typed cell
/// access by position (`NamedRow::cell_as`), and a `From` impl converting
/// the tuple of field values into the struct, so rows can be built
/// positionally with `sortable::row!`:
///
/// ```ignore
/// #[derive(NamedRow)]
/// struct Process {
///     uid: u32,
///     user: String,
///     tty: Option<String>,
/// }
///
/// let mut table = Table::<Process>::default(); // headers: "uid", "user", "tty"
/// table.push(row![3u32, None]); // (u32, Option<String>) -> Process
/// ```
///
/// Currently only plain structs without generics are supported; every field
/// type must implement `sortable::ColumnType`.
#[proc_macro_derive(NamedRow)]
pub fn derive_named_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_named_row(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_named_row(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "NamedRow does not support generic structs",
        ));
    }

    let fields = named_fields(input)?;
    let name = &input.ident;

    let idents: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("named field"))
        .collect();
    let tys: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let names: Vec<String> = idents.iter().map(|i| i.to_string()).collect();
    let indices: Vec<usize> = (0..fields.len()).collect();
    let tuple_indices: Vec<syn::Index> = (0..fields.len()).map(syn::Index::from).collect();

    Ok(quote! {
        impl ::sortable::NamedRow for #name {
            const KINDS: &'static [::sortable::ColumnKind] = &[
                #(<#tys as ::sortable::ColumnType>::KIND),*
            ];

            const NAMES: &'static [&'static str] = &[#(#names),*];

            fn cell_as<T: ::sortable::ColumnType>(
                &self,
                index: usize,
            ) -> ::core::option::Option<&T> {
                #(
                    if index == #indices {
                        return (&self.#idents as &dyn ::core::any::Any)
                            .downcast_ref::<T>();
                    }
                )*
                ::core::option::Option::None
            }
        }

        impl ::core::convert::From<(#(#tys,)*)> for #name {
            /// Converts the tuple of field values (see `sortable::row!`)
            /// into the row struct, in field order.
            fn from(row: (#(#tys,)*)) -> Self {
                Self { #(#idents: row.#tuple_indices),* }
            }
        }
    })
}

fn named_fields(input: &DeriveInput) -> syn::Result<Vec<&syn::Field>> {
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "NamedRow can only be derived for structs",
            ));
        }
    };
    match &data.fields {
        Fields::Named(fields) => Ok(fields.named.iter().collect()),
        other => Err(syn::Error::new_spanned(
            other,
            "NamedRow requires named fields",
        )),
    }
}
