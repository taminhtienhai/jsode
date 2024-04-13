use proc_macro::TokenStream;

pub fn desrialize(tokens: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let syn::DeriveInput { attrs: _attrs, vis, ident, generics, data, } = syn::parse(tokens)?;

    let method_impl = crate::common::parse_body(data)?;

    Ok(quote::quote! {
        impl #generics json_parser::prelude::Deserialize for #ident #generics {
            fn parse(out: &json_parser::prelude::JsonOutput<'_>) -> Result<Self, json_parser::prelude::JsonError> {
                #method_impl
            }
        }
    })
}

pub fn serialize(tokens: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: syn::DeriveInput = syn::parse(tokens)?;
    Ok(quote::quote!())
}