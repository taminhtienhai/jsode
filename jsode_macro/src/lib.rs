use proc_macro::TokenStream;
use quote::{quote, ToTokens};

pub(crate) mod common;
mod derive;


#[proc_macro_derive(Deserialize, attributes(prop))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    match derive::desrialize(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }.into()
}

#[proc_macro_attribute]
pub fn reflection(_attr: TokenStream, src: TokenStream) -> TokenStream {
    let syn::ItemFn { sig, block, attrs, vis } = syn::parse(src).unwrap();
    let syn::Signature { ref ident, ref inputs, .. } = sig;
    let syn::Block { brace_token, stmts } = block.as_ref();
    // let fn_name_stmt = local_str_variable("__fn_ident", ident.as_str());

    let fn_name_stmt: syn::Stmt = syn::parse2(quote!(let __fn_ident: &str = stringify!(#ident);)).unwrap();

    let mut new_stmts = vec![fn_name_stmt];
    for (index, arg) in inputs.iter().enumerate() {
        match arg {
            syn::FnArg::Receiver(_) => {
                let var_ident = format!("__arg_{:o}", index);
                new_stmts.push(local_str_variable(var_ident.as_str(), "\"self\""));
            },
            syn::FnArg::Typed(syn::PatType { pat, .. }) => {
                match &pat.as_ref() {
                    syn::Pat::Ident(ident) => {
                        let var_ident = format!("__arg_{:o}", index);
                        let var_value = format!("{}", ident.ident);
                        new_stmts.push(local_str_variable(var_ident.as_str(), var_value.as_str()));
                    }, 
                    _ => {
                        let var_ident = format!("__arg_{:o}", index);
                        new_stmts.push(local_str_variable(var_ident.as_str(), "\"unknown\""));
                    }
                }
            }
        }
    }
    
    for stmt in stmts {
        new_stmts.push(stmt.clone());
    }

    let body = syn::Block { brace_token: *brace_token, stmts: new_stmts };
    
    syn::ItemFn { sig: sig.clone(), block: Box::new(body), vis, attrs }.into_token_stream().into()
}

// this shit is my first attempt to prepare an local variable
// could be simpler by leverage `parse(..).unwrap()` (note that '..' must be an correct expression, otherwise program will panic)
#[rustfmt::skip]
fn local_str_variable(ident: &str, value: &str) -> syn::Stmt {
    let expr = Box::new(syn::Expr::Lit(syn::ExprLit { attrs: Vec::default(), lit: syn::Lit::Str(syn::LitStr::new(value, proc_macro2::Span::call_site())) })); 
    syn::Stmt::Local(syn::Local {
        attrs: Vec::default(),
        init: Some(syn::LocalInit {
             diverge: None,
             eq_token: syn::token::Eq { ..Default::default() },
             expr,
        }),
        let_token: syn::token::Let { ..Default::default() },
        semi_token: syn::token::Semi { ..Default::default() },
        pat: syn::Pat::Type(syn::PatType {
            attrs: Vec::default(),
            pat: Box::new(syn::Pat::Ident(syn::PatIdent { ident: syn::Ident::new(ident, proc_macro2::Span::call_site()), attrs: Vec::default(), by_ref: None, mutability: None, subpat: None})),
            colon_token: syn::token::Colon { ..Default::default() },
            ty: Box::new(syn::Type::Infer(syn::TypeInfer { underscore_token: syn::token::Underscore { ..Default::default() } })),
        })
    })
}