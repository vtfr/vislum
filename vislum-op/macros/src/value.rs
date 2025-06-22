use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Data, DeriveInput, Expr, Fields, Lit, Meta};

struct ValueAttributes {
    name: String,
    serializable: bool,
}

/// Parse the #[value(name = "name", serializable)] attribute
fn parse_value_attributes(input: &DeriveInput) -> syn::Result<ValueAttributes> {
    let attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("value"))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                input,
                "Missing required #[value(name = \"name\")] attribute",
            )
        })?;

    let meta_list = attr
        .meta
        .require_list()
        .map_err(|_| syn::Error::new_spanned(attr, "Expected #[value(...)] with parameters"))?;

    let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
        .parse2(meta_list.tokens.clone())
        .map_err(|_| syn::Error::new_spanned(attr, "Invalid syntax in #[value(...)] attribute"))?;

    let mut name: Option<String> = None;
    let mut serializable = false;

    for meta in metas {
        if meta.path().is_ident("name") {
            if let Meta::NameValue(name_value) = &meta {
                if let Expr::Lit(expr_lit) = &name_value.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        name = Some(lit_str.value());
                        continue;
                    }
                }
            }
            return Err(syn::Error::new_spanned(
                &meta,
                "Expected 'name' to be name = \"string\"",
            ));
        } else if meta.path().is_ident("serializable") {
            match &meta {
                Meta::NameValue(name_value) => {
                    if let Expr::Lit(expr_lit) = &name_value.value {
                        if let Lit::Bool(lit_bool) = &expr_lit.lit {
                            serializable = lit_bool.value;
                            continue;
                        }
                    }
                }
                Meta::Path(_) => {
                    serializable = true;
                    continue;
                }
                _ => {}
            }
            return Err(syn::Error::new_spanned(
                &meta,
                "Expected 'serializable' to be flag or boolean",
            ));
        }

        return Err(syn::Error::new_spanned(
            &meta,
            "Unknown parameter in #[value(...)] attribute",
        ));
    }

    let name = name.ok_or_else(|| {
        syn::Error::new_spanned(
            input,
            "Missing required 'name' parameter in #[value(name = \"name\")] attribute",
        )
    })?;

    Ok(ValueAttributes { name, serializable })
}

pub fn derive_value_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let attributes = parse_value_attributes(&input)?;

    let type_name = attributes.name;
    let serializable = attributes.serializable;

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "Value can only be derived for enums",
        ));
    };

    // Check that all variants are unit variants
    for variant in &data_enum.variants {
        match &variant.fields {
            Fields::Unit => {}
            Fields::Unnamed(_) | Fields::Named(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Only unit enum variants are supported",
                ));
            }
        }
    }

    // Extract variant names and data for use in quote! macro
    let variant_names: Vec<_> = data_enum.variants.iter().map(|v| &v.ident).collect();
    let variant_name_strs: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect();
    let variant_indices: Vec<_> = (0..variant_names.len()).collect();

    // Generate serialization field based on serializable flag
    let serialization_field = if serializable {
        quote! { Some(vislum_graph::value::ValueTypeSerializationInfo::new::<#name>()) }
    } else {
        quote! { None }
    };

    let expanded = quote! {
        impl TryFrom<vislum_graph::value::TaggedValue> for #name {
            type Error = vislum_graph::value::IncompatibleValueTypeError;

            fn try_from(value: vislum_graph::value::TaggedValue) -> Result<Self, Self::Error> {
                match value {
                    vislum_graph::value::TaggedValue::CustomValue(custom_value) => {
                        match custom_value.as_any().downcast_ref::<#name>() {
                            Some(enum_value) => Ok(*enum_value),
                            None => Err(vislum_graph::value::IncompatibleValueTypeError),
                        }
                    }
                    _ => Err(vislum_graph::value::IncompatibleValueTypeError),
                }
            }
        }

        impl Into<vislum_graph::value::TaggedValue> for #name {
            #[inline(always)]
            fn into(self) -> vislum_graph::value::TaggedValue {
                vislum_graph::value::TaggedValue::CustomValue(
                    vislum_graph::value::CustomValue::new(self)
                )
            }
        }

        impl vislum_graph::value::Value for #name {
            const INFO: vislum_graph::value::ValueTypeInfo = vislum_graph::value::ValueTypeInfo {
                id: vislum_graph::value::ValueTypeId(#type_name),
                variants: Some(&[
                    #(vislum_graph::value::ValueTypeVariantInfo {
                        name: #variant_name_strs,
                        constructor: || {
                            let value = #name::#variant_names;
                            vislum_graph::value::CustomValue::new(value)
                        },
                    },)*
                ]),
                serialization: #serialization_field,
                default_fn: Some(|| <#name as Default>::default().into()),
            };
        }

        impl vislum_graph::value::DynValue for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn clone_custom_value(&self) -> vislum_graph::value::CustomValue {
                vislum_graph::value::CustomValue::new(*self)
            }

            fn variant_index(&self) -> Option<usize> {
                let index = match self {
                    #(#name::#variant_names => #variant_indices,)*
                };
                Some(index)
            }

            fn type_info(&self) -> &'static vislum_graph::value::ValueTypeInfo {
                &<#name as vislum_graph::value::Value>::INFO
            }
        }
    };

    Ok(expanded)
}
