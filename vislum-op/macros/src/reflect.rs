use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parser, Attribute, DeriveInput, Expr, Field, Ident, Lit, LitStr, Meta, Type};

struct Input<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub attrs: InputAttributes,
}

impl Input<'_> {
    pub fn emit_input_info(&self) -> TokenStream {
        let name = self
            .attrs
            .name
            .as_ref()
            .map(|name| quote! { #name })
            .unwrap_or_else(|| {
                let name = self.ident.to_string();
                quote! { #name }
            });

        let default = self
            .attrs
            .default
            .as_ref()
            .map(|default| {
                let ty = self.ty;
                quote! { Some(vislum_op::TaggedValue::from(#default)) }
            })
            .unwrap_or_else(|| quote! { None });

        quote! {
            vislum_op::InputInfo {
                name: #name,
                description: None,
                default_value: #default,
                ..Default::default()
            }
        }
    }
}

#[derive(Default)]
struct InputAttributes {
    name: Option<LitStr>,
    default: Option<Expr>,
}

impl InputAttributes {
    fn parse(attr: &Attribute) -> syn::Result<Self> {
        let meta_list = match &attr.meta {
            Meta::Path(_) => return Ok(Default::default()),
            Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Invalid syntax in #[input(...)] attribute",
                ))
            }
            Meta::List(meta_list) => meta_list,
        };

        let mut name = None::<LitStr>;
        let mut default = None::<Expr>;
        let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            .parse2(meta_list.tokens.clone())?;

        for meta in metas {
            if meta.path().is_ident("name") {
                let value: LitStr = meta.require_list()?.parse_args()?;
                name = Some(value);
            } else if meta.path().is_ident("default") {
                let value: Expr = meta.require_list()?.parse_args()?;
                default = Some(value);
            }
        }

        Ok(Self { name, default })
    }
}

impl Input<'_> {
    fn from_field(field: &Field) -> syn::Result<Input> {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("input"))
            .expect("Missing required #[input] attribute");

        let attrs = InputAttributes::parse(&attr)?;
        Ok(Input { ident, ty, attrs })
    }
}

struct Output<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub attrs: OutputAttributes,
}

impl Output<'_> {
    pub fn from_field(field: &Field) -> syn::Result<Output> {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("output"))
            .unwrap();

        let attrs = OutputAttributes::parse(&attr)?;
        Ok(Output { ident, ty, attrs })
    }

    pub fn emit_output_info(&self) -> TokenStream {
        let name = self
            .attrs
            .name
            .as_ref()
            .map(|name| quote! { #name })
            .unwrap_or_else(|| {
                let name = self.ident.to_string();
                quote! { #name }
            });

        quote! {
            vislum_op::OutputInfo {
                name: #name,
                description: None,
            }
        }
    }
}

#[derive(Default)]
struct OutputAttributes {
    name: Option<LitStr>,
    description: Option<LitStr>,
}

impl OutputAttributes {
    fn parse(attr: &Attribute) -> syn::Result<Self> {
        let meta_list = match &attr.meta {
            Meta::Path(_) => return Ok(Default::default()),
            Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Invalid syntax in #[output(...)] attribute",
                ))
            }
            Meta::List(meta_list) => meta_list,
        };

        let mut name = None::<LitStr>;
        let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            .parse2(meta_list.tokens.clone())?;

        for meta in metas {
            if meta.path().is_ident("name") {
                let value: LitStr = meta.require_list()?.parse_args()?;
                name = Some(value);
            }
        }

        Ok(Self {
            name,
            description: None,
        })
    }
}

#[derive(Clone)]
struct State<'a> {
    ident: &'a Ident,
}

impl State<'_> {
    fn from_field(field: &Field) -> syn::Result<State> {
        let ident = field.ident.as_ref().unwrap();
        Ok(State { ident })
    }
}

fn parse_fields<'a>(
    fields: impl Iterator<Item = &'a Field>,
) -> syn::Result<(Vec<Input<'a>>, Vec<Output<'a>>, Vec<State<'a>>)> {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut states = Vec::new();

    for field in fields {
        if field.attrs.iter().any(|attr| attr.path().is_ident("input")) {
            inputs.push(Input::from_field(field)?);
        } else if field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("output"))
        {
            outputs.push(Output::from_field(field)?);
        } else {
            states.push(State::from_field(field)?);
        }
    }

    Ok((inputs, outputs, states))
}

pub fn derive_reflect_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    // Extract fields
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Reflect derive macro only supports structs",
            ))
        }
    };

    let (inputs, outputs, states) = parse_fields(fields.iter())?;

    // Generate the struct name as a string for the node type ID
    let operator_type_id = ident.to_string();

    // Generate input initializations using the helper function
    let input_constructors = inputs.iter().map(|input| {
        let ident = input.ident;
        let info = input.emit_input_info();
        let ty = input.ty;
        quote! {
            #ident: <#ty as vislum_op::ConstructInput>::construct_input(#info),
        }
    });

    let output_constructors = outputs.iter().map(|output| {
        let ident = output.ident;
        let info = output.emit_output_info();
        let ty = output.ty;
        quote! {
            #ident: <#ty as vislum_op::ConstructOutput>::construct_output(#info),
        }
    });

    let outputs_len = outputs.len();
    let output_idents = outputs
        .iter()
        .map(|output| output.ident)
        .collect::<Vec<_>>();
    let output_indexes = (0..outputs.len()).collect::<Vec<_>>();

    let input_len = inputs.len();
    let input_idents = inputs.iter().map(|input| input.ident).collect::<Vec<_>>();
    let input_indexes = (0..input_len).collect::<Vec<_>>();
    
    let input_specifications = inputs.iter().map(|input| {
        let name: String = input.attrs.name.clone()
            .map(|name| name.value())
            .unwrap_or_else(|| input.ident.to_string().into());

        let ty = input.ty;
        
        quote! {
            vislum_op::InputSpecification {
                name: #name.into(),
                value_type: <#ty as vislum_op::ConstructInput>::type_info(),
            }
        }
    });

    let state_idents = states.iter().map(|state| state.ident);

    let result = quote! {
        impl vislum_op::Reflect for #ident {
            fn type_id(&self) -> vislum_op::OperatorTypeId {
                vislum_op::OperatorTypeId::new(#operator_type_id)
            }

            fn num_inputs(&self) -> usize {
                #input_len
            }

            fn num_outputs(&self) -> usize {
                #outputs_len
            }

            fn get_input(&self, index: vislum_op::InputIndex) -> Option<&dyn vislum_op::InputReflect> {
                match index {
                    #(#input_indexes => Some(&self.#input_idents),)*
                    _ => None,
                }
            }

            fn get_input_mut(&mut self, index: vislum_op::InputIndex) -> Option<&mut dyn vislum_op::InputReflect> {
                match index {
                    #(#input_indexes => Some(&mut self.#input_idents),)*
                    _ => None,
                }
            }

            fn get_output(&self, index: vislum_op::OutputIndex) -> Option<&dyn vislum_op::OutputReflect> {
                match index {
                    #(#output_indexes => Some(&self.#output_idents),)*
                    _ => None,
                }
            }
        }

        impl vislum_op::ConstructOperator for #ident {
            fn construct_operator() -> Box<dyn vislum_op::Operator> {
                Box::new(#ident {
                    #(#input_constructors)*
                    #(#output_constructors)*
                    #(#state_idents: Default::default(),)*
                })
            }
        }

        impl vislum_op::RegisterOperator for #ident {
            fn register_operator(registry: &mut vislum_op::OperatorTypeRegistry) {
                registry.add(vislum_op::OperatorType {
                    id: vislum_op::OperatorTypeId::new(#operator_type_id),
                    inputs: vec![
                        #(#input_specifications),*
                    ],
                    construct: <#ident as vislum_op::ConstructOperator>::construct_operator,
                });
            }
        }
    };

    Ok(result)
}
