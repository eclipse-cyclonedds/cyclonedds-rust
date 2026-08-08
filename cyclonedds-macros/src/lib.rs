//! Derive macros for the `Topicable` and `FixedSize` traits.

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

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

            let key_name = format!("Key<{ident}>");
            let key_field_names = keys.iter().map(|f| &f.ident);
            tokens.extend(quote! {
                #[allow(non_snake_case)]
                #[doc(hidden)]
                mod #key_mod {
                    #[doc(hidden)]
                    #[derive(Default, serde::Serialize, serde::Deserialize, Clone, PartialEq, Hash)]
                    pub struct Key {
                        #(#key_field_defs),*
                    }

                    impl std::fmt::Debug for Key {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            let mut debug = f.debug_struct(#key_name);
                            #(
                                debug.field(stringify!(#key_field_names), &self.#key_field_names);
                            )*
                            debug.finish()
                        }
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
            });
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
/// `<Name>Key` struct that implements `CdrBounds` and [`Default`]. Structs
/// with no `#[dds(key)]` fields use [`()`](primitive@unit) as their key type.
/// The topic struct must also implement [`Default`].
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
    TopicableAttributes::from_derive_input(&input)
        .map(|topicable| topicable.to_token_stream().into())
        .unwrap_or_else(|e| e.write_errors().into())
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named, struct_tuple), forward_attrs(repr))]
struct FixedSizeAttributes {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), Field>,

    /// The `repr` attributes, forwarded so the derive can insist on `repr(C)`.
    attrs: Vec<syn::Attribute>,
}

/// Whether any `repr` attribute names `word` at its top level.
///
/// So `repr(C)`, `repr(C, packed)` and `repr(align(8), C)` all report `C`, while
/// `repr(align(8))` and `repr(transparent)` do not.
fn has_repr(attributes: &[syn::Attribute], word: &str) -> bool {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("repr"))
        .any(|attribute| {
            let syn::Meta::List(list) = &attribute.meta else {
                return false;
            };

            list.tokens.clone().into_iter().any(
                |token| matches!(token, proc_macro2::TokenTree::Ident(ref ident) if ident == word),
            )
        })
}

/// Whether any `repr` attribute names the `C` representation.
fn has_repr_c(attributes: &[syn::Attribute]) -> bool {
    has_repr(attributes, "C")
}

impl FixedSizeAttributes {
    /// Produce the implementation, or an error explaining why the type cannot
    /// have one.
    fn expand(&self) -> darling::Result<proc_macro2::TokenStream> {
        let Self {
            ref ident,
            ref generics,
            ref data,
            ref attrs,
        } = *self;

        // A generic type has no single layout, so there is nothing to promise.
        if !generics.params.is_empty() {
            return Err(darling::Error::custom(
                "`FixedSize` cannot be derived for a generic type: its layout has to be fixed",
            )
            .with_span(generics));
        }

        // `packed` changes the layout away from what a C compiler produces for
        // the equivalent IDL type, and forces alignment to 1, which would make
        // the alignment assertion below vacuous.
        if has_repr(attrs, "packed") {
            return Err(darling::Error::custom(
                "`FixedSize` cannot be derived for a `#[repr(packed)]` type: its layout is not \
                 what a C compiler produces for the equivalent IDL type",
            )
            .with_span(ident));
        }

        if !has_repr_c(attrs) {
            return Err(darling::Error::custom(
                "`FixedSize` requires `#[repr(C)]`: a `repr(Rust)` type has no guaranteed layout, \
                 so its bytes mean nothing to another process",
            )
            .with_span(ident));
        }

        let field_types = data
            .as_ref()
            .take_struct()
            .expect("darling is configured to accept structs only")
            .fields
            .into_iter()
            .map(|field| &field.ty)
            .collect::<Vec<_>>();

        // Each field's type has to carry the same guarantees. Spanned at the
        // field so the error points there rather than at the derive.
        let field_assertions = field_types.iter().map(|ty| {
            quote_spanned! { ty.span() => __assert_fixed_size::<#ty>(); }
        });

        // A `repr(C)` struct whose size exceeds the sum of its fields' sizes has
        // padding somewhere, which would travel uninitialized.
        let size_of_fields = if field_types.is_empty() {
            quote!(0)
        } else {
            let sizes = field_types
                .iter()
                .map(|ty| quote!(::core::mem::size_of::<#ty>()));
            quote!(#(#sizes)+*)
        };

        Ok(quote! {
            #[automatically_derived]
            #[allow(unsafe_code)]
            // SAFETY: the assertions below check every part of the contract that
            // can be checked: `repr(C)`, that each field is itself `FixedSize`
            // (and so transitively `Copy`, ruling out indirection), that there is
            // no padding, and that the alignment stays within what a PSMX
            // implementation guarantees for a loan.
            unsafe impl ::cyclonedds::FixedSize for #ident {}

            const _: () = {
                const fn __assert_fixed_size<T: ::cyclonedds::FixedSize>() {}
                #(#field_assertions)*

                assert!(
                    ::core::mem::size_of::<#ident>() == #size_of_fields,
                    "cannot derive `FixedSize`: the type has padding bytes, which would be sent \
                     uninitialized. Reorder the fields largest-first, or pad explicitly with a \
                     byte array."
                );
                assert!(
                    ::core::mem::size_of::<#ident>() > 0,
                    "cannot derive `FixedSize`: the type is zero-sized, and Cyclone DDS cannot \
                     loan a zero-length sample."
                );
                assert!(
                    ::core::mem::align_of::<#ident>() <= 8,
                    "cannot derive `FixedSize`: the type needs more than 8 byte alignment, which \
                     a PSMX implementation is not required to provide for a loan."
                );
            };
        })
    }
}

/// Derives [`FixedSize`], marking a type as safe to share with another process
/// verbatim.
///
/// Requires `#[repr(C)]`, and the type must be [`Copy`] because `FixedSize`
/// requires it.
///
/// # Examples
///
/// ```ignore
/// #[derive(cyclonedds::FixedSize, Clone, Copy)]
/// #[repr(C)]
/// struct Reading {
///     sensor_id: u32,
///     value: f64,
/// }
/// ```
///
/// # Panics
///
/// Fails at compile time when the type is an enum, a union, generic, or not
/// `#[repr(C)]`; when any field's type is not itself `FixedSize`; when the layout
/// has padding; when the type is zero-sized; or when it needs more than 8 byte
/// alignment.
#[proc_macro_derive(FixedSize)]
pub fn derive_fixed_size(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    FixedSizeAttributes::from_derive_input(&input)
        .and_then(|fixed_size| fixed_size.expand())
        .map(Into::into)
        .unwrap_or_else(|error| error.write_errors().into())
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

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
    fn test_has_repr_c_recognizes_the_c_representation() {
        // The `C` may be alone, or alongside other modifiers in either order.
        for attributes in [
            quote!(#[repr(C)]),
            quote!(#[repr(C, packed)]),
            quote!(#[repr(align(8), C)]),
        ] {
            let input: syn::DeriveInput = syn::parse_quote! {
                #attributes
                struct Data {
                    x: u32,
                }
            };
            assert!(
                has_repr_c(&input.attrs),
                "expected `{attributes}` to count as `repr(C)`"
            );
        }

        // These do not fix the layout to the C one.
        for attributes in [
            quote!(#[repr(transparent)]),
            quote!(#[repr(align(8))]),
            quote!(#[derive(Clone)]),
        ] {
            let input: syn::DeriveInput = syn::parse_quote! {
                #attributes
                struct Data {
                    x: u32,
                }
            };
            assert!(
                !has_repr_c(&input.attrs),
                "expected `{attributes}` not to count as `repr(C)`"
            );
        }
    }

    #[test]
    fn test_fixed_size_requires_repr_c() {
        let input = parse_quote! {
            struct Data {
                x: u32,
            }
        };
        let error = FixedSizeAttributes::from_derive_input(&input)
            .unwrap()
            .expand()
            .unwrap_err();
        assert!(
            format!("{error}").contains("requires `#[repr(C)]`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_fixed_size_rejects_generic_types() {
        let input = parse_quote! {
            #[repr(C)]
            struct Data<T> {
                value: T,
            }
        };
        let error = FixedSizeAttributes::from_derive_input(&input)
            .unwrap()
            .expand()
            .unwrap_err();
        assert!(
            format!("{error}").contains("generic"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_fixed_size_expands_for_a_repr_c_struct() {
        let input = parse_quote! {
            #[repr(C)]
            struct Data {
                x: u32,
                y: f64,
            }
        };
        let expanded = FixedSizeAttributes::from_derive_input(&input)
            .unwrap()
            .expand()
            .unwrap()
            .to_string();

        assert!(expanded.contains("unsafe impl"), "{expanded}");
        // Both field types have to be asserted, and the padding and alignment
        // checks have to be present.
        assert!(
            expanded.contains("__assert_fixed_size :: < u32 > ()"),
            "{expanded}"
        );
        assert!(
            expanded.contains("__assert_fixed_size :: < f64 > ()"),
            "{expanded}"
        );
        assert!(expanded.contains("size_of"), "{expanded}");
        assert!(expanded.contains("align_of"), "{expanded}");
    }

    #[test]
    fn test_fixed_size_accepts_a_tuple_struct() {
        let input = parse_quote! {
            #[repr(C)]
            struct Data(u32, u32);
        };
        assert!(
            FixedSizeAttributes::from_derive_input(&input)
                .unwrap()
                .expand()
                .is_ok()
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
