use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Expr, Ident, Meta,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
};

pub enum Provides {
    Map { feature: Ident, from: Ident },
    Override { feature: Ident, value: Expr },
}

impl Parse for Provides {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let feature: Ident = input.parse()?;

        if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            let value: Expr = input.parse()?;

            Ok(Self::Override { feature, value })
        } else if input.peek(syn::Token![=>]) {
            input.parse::<syn::Token![=>]>()?;
            let from = input.parse()?;

            Ok(Self::Map { feature, from })
        } else {
            Ok(Self::Map {
                from: feature.clone(),
                feature,
            })
        }
    }
}

#[derive(Default)]
pub struct WiringSpecification {
    pub base: bool,
    pub version: Option<Expr>,
    pub promoted: Option<Expr>,
    pub extension: Option<Ident>,
    pub provides: Vec<Provides>,
}

impl WiringSpecification {
    pub fn supported(&self) -> TokenStream {
        if let Some(version) = &self.version {
            return quote! {
                api_version >= #version
            };
        }

        match (&self.promoted, &self.extension) {
            (Some(promoted), Some(extension)) => {
                quote! {
                    api_version < #promoted && device_extensions.#extension
                }
            }
            (None, Some(extension)) => {
                quote! {
                    device_extensions.#extension
                }
            }
            _ => quote! { false },
        }
    }
}

impl WiringSpecification {
    pub fn from_attr(attr: &Attribute) -> syn::Result<Self> {
        let meta_list = attr.meta.require_list()?;
        let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            .parse2(meta_list.tokens.clone())?;

        let mut base = false;
        let mut version = None;
        let mut promoted = None;
        let mut extension = None;
        let mut provides = Vec::<Provides>::new();

        for meta in metas {
            if meta.path().is_ident("base") {
                base = true;
                break;
            } else if meta.path().is_ident("version") {
                let item_meta = meta.require_list()?;
                let value = item_meta.parse_args()?;
                version = Some(value);
            } else if meta.path().is_ident("promoted") {
                let item_meta = meta.require_list()?;
                let value = item_meta.parse_args()?;
                promoted = Some(value);
            } else if meta.path().is_ident("extension") {
                let item_meta = meta.require_list()?;
                let value = item_meta.parse_args()?;
                extension = Some(value);
            } else if meta.path().is_ident("provides") {
                let item_meta = meta.require_list()?;
                let value = item_meta
                    .parse_args_with(Punctuated::<Provides, syn::Token![,]>::parse_terminated)?;
                provides.extend(value);
            }
        }

        Ok(Self {
            base,
            version,
            promoted,
            extension,
            provides,
        })
    }
}

pub fn derive_wiring_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let data_struct = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Wiring derive macro only supports structs",
            ));
        }
    };

    let mut fields = Vec::with_capacity(data_struct.fields.len());

    for field in data_struct.fields.iter() {
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("wiring"));

        let spec = match attr {
            Some(attr) => WiringSpecification::from_attr(&attr)?,
            None => WiringSpecification::default(),
        };

        fields.push((field.ident.as_ref().unwrap(), spec));
    }

    let wire_to_physical_features2 = codegen_wire_to_physical_features2(&fields);
    let supported_features = codegen_supported_features(&fields);
    let wire_to_device_create_info = codegen_wire_to_device_create_info(&fields);

    Ok(quote! {
        impl #ident {
            pub fn wire_to_physical_features2<'a>(
                &'a mut self,
                api_version: Version,
                device_extensions: &DeviceExtensions,
                mut physical_device_features2: vk::PhysicalDeviceFeatures2<'a>,
            ) -> vk::PhysicalDeviceFeatures2<'a> {
                #wire_to_physical_features2
            }

            pub fn supported_features(&self) -> DeviceFeatures {
                #supported_features
            }

            pub fn wire_to_device_create_info<'a>(
                &'a mut self,
                api_version: Version,
                device_extensions: &DeviceExtensions,
                device_features: &DeviceFeatures,
                mut device_create_info: vk::DeviceCreateInfo<'a>,
            ) -> vk::DeviceCreateInfo<'a> {
                #wire_to_device_create_info
            }
        }
    })
}

/// Generates the read implementation for the struct.
///
/// Assumes the following variables are in scope:
/// - `api_version: Version`
/// - `device_extensions: &DeviceExtensions`
/// - `mut create_info: vk::PhysicalDeviceFeatures2<'a>`
pub fn codegen_wire_to_physical_features2<'a>(
    fields: &[(&'a Ident, WiringSpecification)],
) -> TokenStream {
    let mut tokens = TokenStream::new();

    tokens.extend(fields.iter().map(|(field_ident, spec)| {
        if spec.base {
            quote! {
                physical_device_features2 = physical_device_features2
                    .features(&mut self.#field_ident);
            }
        } else {
            let supported = spec.supported();
            quote! {
                if #supported {
                    let mut features = self.#field_ident.get_or_insert_default();
                    physical_device_features2 = physical_device_features2.push_next(features);
                }
            }
        }
    }));

    tokens.extend(quote! {
        physical_device_features2
    });

    tokens
}

/// Generates the implementation of the `enabled_features` method for the struct.
///
/// Assumes the following variables are in scope:
/// - `api_version: Version`
/// - `device_extensions: &DeviceExtensions`
/// - `mut physical_device_features2: vk::PhysicalDeviceFeatures2<'a>`
pub fn codegen_supported_features<'a>(fields: &[(&'a Ident, WiringSpecification)]) -> TokenStream {
    let mut tokens = quote! {
        let mut supported_features: DeviceFeatures = DeviceFeatures::default();
    };

    for (field_ident, spec) in fields {
        let provides = spec.provides.iter().map(|provide| match provide {
            Provides::Map { feature, from } => {
                quote! {
                    supported_features.#feature = features.#from == vk::TRUE;
                }
            }
            Provides::Override { feature, value } => {
                quote! {
                    supported_features.#feature = #value;
                }
            }
        });

        tokens.extend(quote! {
            if let Some(features) = &self.#field_ident {
                #( #provides )*
            }
        });
    }

    tokens.extend(quote! {
        supported_features
    });

    tokens
}

/// Generates the implementation of the `write` method for the struct.
///
/// Assumes the following variables are in scope:
/// - `api_version: Version`
/// - `device_features: &DeviceFeatures`
/// - `device_extensions: &DeviceExtensions`
/// - `mut device_create_info: DeviceCreateInfo`
pub fn codegen_wire_to_device_create_info<'a>(
    fields: &[(&'a Ident, WiringSpecification)],
) -> TokenStream {
    let mut tokens = TokenStream::new();

    tokens.extend(quote! {
        fn to_vk(value: bool) -> vk::Bool32 {
            if value {
                vk::TRUE
            } else {
                vk::FALSE
            }
        }
    });

    for (field_ident, spec) in fields {
        let provides = spec.provides.iter().filter_map(|provide| match provide {
            Provides::Map { feature, from } => Some(quote! {
                feature.#from = to_vk(device_features.#feature);
            }),
            _ => None,
        });

        if spec.base {
            tokens.extend(quote! {
                {
                    let mut feature = &mut device_create_info.features;
                    #( #provides )*
                }
            });
        } else {
            let supported = spec.supported();
            tokens.extend(quote! {
                if #supported {
                    let mut feature = self.#field_ident.get_or_insert_default();
                    #( #provides )*
                    device_create_info = device_create_info.push_next(feature);
                }
            });
        }
    }

    tokens.extend(quote! {
        device_create_info
    });

    tokens
}
