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
    name: Option<syn::Ident>,
    ty: syn::Type,
}

impl BodyProp {
    pub fn new(name: Option<syn::Ident>, ty: syn::Type) -> Self {
        Self { name, ty, }
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
                for BodyProp { name, ty } in props {
                    let item = match check_type(ty) {
                        FieldType::Primitive => quote::quote! {
                            #name: jsode::prelude::JsonPsr::parse_into::<#ty>(
                                &jsode::prelude::JsonIdx::index(out, stringify!(#name))
                                .ok_or_else(|| jsode::prelude::JsonError::custom(format!("key not found: `{}`", stringify!(#name)), jsode::prelude::Span::default()))?
                            )?,
                        },
                        FieldType::Option(inner_type) => quote::quote! {
                            #name: jsode::prelude::JsonIdx::index(out, stringify!(#name))
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
    use syn::{DataEnum, DataStruct};
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
                .map(|syn::Field { ident, ty, .. }| BodyProp::new(ident, ty))
                .collect::<Vec<_>>();
        StructType::Struct(StructBody { props })
    }

    fn parse_struct_unnamed( syn::FieldsUnnamed { unnamed, .. }: syn::FieldsUnnamed) -> StructType {
        let props = unnamed.into_iter()
                .map(|syn::Field { ident, ty, .. }| BodyProp::new(ident, ty))
                .collect::<Vec<_>>();
        StructType::Tuple(StructBody { props })
    }

    pub fn parse_enum_body(DataEnum { variants, .. }: DataEnum) -> impl ToTokens {
        quote::quote!()
    }
}