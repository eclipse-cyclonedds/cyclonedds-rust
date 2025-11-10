//! Derive macro for the `Topicable` trait.

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};

#[derive(Debug, FromField)]
#[darling(attributes(dds))]
struct Field {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    key: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(dds), supports(struct_named))]
struct TopicableAttributes {
    ident: syn::Ident,

    data: darling::ast::Data<(), Field>,

    type_name: Option<String>,
}

impl ToTokens for TopicableAttributes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let TopicableAttributes {
            ref ident,
            ref data,
            ref type_name,
        } = *self;

        let keys = data
            .as_ref()
            .take_struct()
            .expect("the topicable attribute only accepts structs")
            .fields
            .into_iter()
            .filter(|field| field.key)
            .collect::<Vec<_>>();

        let (key_type, from_key, as_key) = if keys.is_empty() {
            (
                quote!(()),
                quote! {
                    fn from_key((): &Self::Key) -> Self {
                       Self::default()
                    }
                },
                quote! {
                    fn as_key(&self) -> Self::Key {}
                },
            )
        } else {
            let key_mod = format_ident!("__cyclonedds_topicable_{}", ident);
            let key_field_defs = keys.iter().map(|f| {
                let n = &f.ident;
                let t = &f.ty;
                quote! { pub #n: #t }
            });
            let key_field_inits = keys.iter().map(|f| {
                let n = &f.ident;
                quote! { #n: self.#n.clone() }
            });
            let key_field_from_key = keys.iter().map(|f| {
                let n = &f.ident;
                quote! { #n: key.#n.clone() }
            });
            let key_size_sum = keys.iter().map(|f| {
                let t = &f.ty;
                quote! {
                    <#t as ::cyclonedds::cdr_bounds::CdrBounds>::max_serialized_cdr_size()
                }
            });
            let key_alignment_max = keys.iter().map(|f| {
                let t = &f.ty;
                quote! {
                    <#t as ::cyclonedds::cdr_bounds::CdrBounds>::alignment()
                }
            });

            tokens.extend(
                quote! {
                    #[allow(non_snake_case)]
                    #[doc(hidden)]
                    mod #key_mod {
                        #[doc(hidden)]
                        #[derive(Default, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Hash)]
                        pub struct Key {
                            #(#key_field_defs),*
                        }

                        impl ::cyclonedds::cdr_bounds::CdrBounds for Key {
                            fn max_serialized_cdr_size() -> ::cyclonedds::cdr_bounds::CdrSize {
                                #(#key_size_sum)+*
                            }
                            fn alignment() -> usize {
                                0 #(.max(#key_alignment_max))*
                            }
                        }
                    }
                }
            );
            (
                quote!(#key_mod::Key),
                quote! {
                    fn from_key(key: &Self::Key) -> Self {
                        Self {
                            #(#key_field_from_key),*,
                            ..Default::default()
                        }
                    }
                },
                quote! {
                    fn as_key(&self) -> Self::Key {
                        Self::Key {
                            #(#key_field_inits),*
                        }
                    }
                },
            )
        };

        let dds_type_name = type_name.as_ref().map(|type_name| {
            quote! {
                fn dds_type_name() -> impl AsRef<str> {
                    #type_name
                }
            }
        });

        tokens.extend(quote! {
            impl ::cyclonedds::Topicable for #ident {
                type Key = #key_type;

                #from_key

                #as_key

                #dds_type_name
            }
        });
    }
}

/// Derives `Topicable` for a named-field struct.
///
/// Fields annotated with `#[dds(key)]` are collected into a generated
/// `<Name>Key` struct that implements `CdrBounds`. Structs with no
/// `#[dds(key)]` fields use [`()`](primitive@unit) as their key type and must
/// implement [`Default`].
///
/// An optional `#[dds(type_name = "...")]` attribute overrides the DDS type
/// name used for topic matching. Without it, the Rust type name is used.
///
/// # Examples
///
/// ```ignore
/// #[derive(cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
/// #[dds(type_name = "MySensor")]
/// pub struct Sensor {
///     #[dds(key)]
///     pub id: u32,
///     pub value: f32,
/// }
///
/// // NOTE: this results in a hidden module being generated `__cyclonedds_topicable_Sensor`
/// // containing the following type: `Key { pub id: u32 }`
/// ```
///
/// # Panics
///
/// Panics at compile time if applied to an enum, a union, a tuple struct, or
/// if `#[dds(type_name)]` is not a valid string literal.
#[proc_macro_derive(Topicable, attributes(dds))]
pub fn derive_topicable(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let topicable = match TopicableAttributes::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    topicable.to_token_stream().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn test_derive_parses_key_fields() {
        let input = parse_quote! {
            struct Sensor {
                #[dds(key)]
                pub id1: u32,
                pub value: f32,
                #[dds(key)]
                pub id2: u32,
            }
        };
        let attributes = TopicableAttributes::from_derive_input(&input).unwrap();
        let fields = attributes.data.take_struct().unwrap();
        let keys: Vec<_> = fields.fields.iter().filter(|f| f.key).collect();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].ident.as_ref().unwrap().to_string(), "id1");
        assert_eq!(keys[1].ident.as_ref().unwrap().to_string(), "id2");
    }

    #[test]
    fn test_derive_parses_type_name() {
        let input = parse_quote! {
            #[dds(type_name = "MySensor")]
            struct Sensor {
                pub id1: u32,
                pub value: f32,
            }
        };
        let attributes = TopicableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attributes.type_name.as_deref(), Some("MySensor"));
    }

    #[test]
    fn test_derive_double_type_name_fails() {
        let input = parse_quote! {
            #[dds(type_name = "MySensor1")]
            #[dds(type_name = "MySensor2")]
            struct Sensor {
                pub id1: u32,
                pub value: f32,
            }
        };
        let error = TopicableAttributes::from_derive_input(&input).unwrap_err();
        assert_eq!(
            format!("{error}"),
            format!("{}", darling::Error::duplicate_field("type_name"))
        );
    }

    #[test]
    fn test_derive_reject_tuple_struct() {
        let input = parse_quote! {
            struct Sensor ( pub u32, pub f32, pub u32 );
        };
        let error = TopicableAttributes::from_derive_input(&input).unwrap_err();
        assert_eq!(
            format!("{error}"),
            format!(
                "{}",
                darling::Error::unsupported_shape_with_expected("unnamed fields", &"named fields")
            )
        );
    }
}
