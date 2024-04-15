use quote::ToTokens;
use syn::Data;

pub fn parse_body(body: Data) -> syn::Result<impl ToTokens> {
    match body {
        Data::Struct(struct_body) => private::parse_struct_body(struct_body),
        Data::Enum(_) => Err(syn::Error::new(proc_macro2::Span::call_site(), "not apply on Enum")),
        Data::Union(_) => Err(syn::Error::new(proc_macro2::Span::call_site(), "not apply on Union")),
    }
}


pub struct BodyProp {
    attrs: Vec<syn::Attribute>,
    name: Option<syn::Ident>,
    ty: syn::Type,
}

impl BodyProp {
    pub fn new(name: Option<syn::Ident>, ty: syn::Type, attrs: Vec<syn::Attribute>) -> Self {
        Self { name, ty, attrs, }
    }
}

/**
 * ```
 * impl Deserialize for $struct_ident {
 *     pub fn parse(parer: &JsonParser, ast: &JsonType) -> Self {
 *         Self {
 *             $prop01: parser.index(ast).parse::<$ty01>(),
 *             $prop02: parser.index(ast).parse::<$ty02>(),
 *             $prop03: parser.index(ast).parse::<$ty03>(),
 *         }
 *     }
 * }
 * ```
 */
pub struct StructBody {
    props: Vec<BodyProp>,
}

pub enum StructType {
    Struct(StructBody),
    Tuple(StructBody),
}

enum FieldType<'ty> {
    Primitive,
    Option(&'ty syn::Type),
    Phantom,
}

fn check_type(ty: &syn::Type) -> FieldType {
    use syn::{
        AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, PathSegment, TypePath,
    };
    let syn::Type::Path(TypePath {
        path: Path {
            leading_colon,
            segments,
        },
        ..
    }) = ty else {
        return FieldType::Primitive;
    };

    if leading_colon.is_none() && segments.len() == 1 {
        if let Some(PathSegment {
            ident,
            arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
        }) = segments.first() {
            if let (1, Some(GenericArgument::Type(inner_type))) = (args.len(), args.first()) {
                if ident == "Option" {
                    return FieldType::Option(inner_type);
                } else if ident == "PhantomData" {
                    return FieldType::Phantom;
                }
            }
        }
    }

    FieldType::Primitive
}

impl ToTokens for StructType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let result = match self {
            Self::Struct(StructBody { props }) => {
                let mut constructor = proc_macro2::TokenStream::new();
                for BodyProp { name, ty, attrs } in props {
                    let prop_index = private::extract_prop_meta(attrs)
                        .or_else(|| Some(private::convert_to_list_str(name.as_ref().unwrap())));
                    let item = match check_type(ty) {
                        FieldType::Primitive => quote::quote! {
                            #name: jsode::prelude::JsonPsr::parse_into::<#ty>(
                                &jsode::prelude::JsonIdx::index(out, #prop_index)
                                .ok_or_else(|| jsode::prelude::JsonError::custom(format!("key not found: `{}`", #prop_index), jsode::prelude::Span::default()))?
                            )?,
                        },
                        FieldType::Option(inner_type) => quote::quote! {
                            #name: jsode::prelude::JsonIdx::index(out, #prop_index)
                                .map(|x| jsode::prelude::JsonPsr::parse_into::<#inner_type>(&x))
                                .map_or(Ok(None), |x| x.map(Some))?,
                        },
                        FieldType::Phantom => quote::quote! {
                            #name: PhantomData::default(),
                        },
                    };
                    constructor.extend(item);
                }

                quote::quote! {
                    Ok(Self {
                        #constructor
                    })
                }
            },
            Self::Tuple(_) => quote::quote!(),
        };
        tokens.extend(result)
    }
}

mod private {
    use proc_macro2::Span;
    use quote::ToTokens;
    use syn::DataStruct;
    use super::StructType;

    use crate::common::{StructBody, BodyProp};


    pub fn parse_struct_body(DataStruct { fields, .. }: DataStruct) -> syn::Result<impl ToTokens> {
        match fields {
            syn::Fields::Named(named) => Ok(parse_struct_named(named)),
            syn::Fields::Unnamed(unnamed) => Ok(parse_struct_unnamed(unnamed)),
            syn::Fields::Unit => Err(syn::Error::new(Span::call_site(), "not support on Unit struct")),
        }
    }

    fn parse_struct_named(syn::FieldsNamed { named, .. }: syn::FieldsNamed) -> StructType {
        let props = named.into_iter()
                .map(|syn::Field { ident, ty, attrs, .. }| BodyProp::new(ident, ty, attrs))
                .collect::<Vec<_>>();
        StructType::Struct(StructBody { props })
    }

    fn parse_struct_unnamed( syn::FieldsUnnamed { unnamed, .. }: syn::FieldsUnnamed) -> StructType {
        let props = unnamed.into_iter()
                .map(|syn::Field { ident, ty, attrs,.. }| BodyProp::new(ident, ty, attrs))
                .collect::<Vec<_>>();
        StructType::Tuple(StructBody { props })
    }

    pub fn extract_prop_meta(attrs: &Vec<syn::Attribute>) -> Option<syn::LitStr> {
        for syn::Attribute { meta, .. } in attrs {
            match meta {
                syn::Meta::NameValue(syn::MetaNameValue { value: syn::Expr::Lit(syn::ExprLit { lit, .. }), .. }) => if let syn::Lit::Str(lit_str) = lit {
                    return Some(lit_str.clone());
                } else {
                    continue;
                },
                _ => continue,
            };
        }
        None
    }

    pub fn convert_to_list_str(ident: &syn::Ident) -> syn::LitStr {
        let ident_str = ident.to_string();
        syn::LitStr::new(&ident_str, ident.span())
    }
}