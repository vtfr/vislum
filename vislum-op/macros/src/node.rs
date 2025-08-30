use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, Field, Ident, Lit, LitStr, Meta, Type, parse::Parser, punctuated::Punctuated};

struct Input<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub attrs: InputAttributes,
}

#[derive(Default)]
struct InputAttributes {
    name: Option<LitStr>,
    default: Option<Expr>,
    assignments: Vec<Ident>,
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
        let mut assignments = Vec::new();
        let metas = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            .parse2(meta_list.tokens.clone())?;

        for meta in metas {
            if meta.path().is_ident("name") {
                let value: LitStr = meta.require_list()?.parse_args()?;
                name = Some(value);
            } else if meta.path().is_ident("default") {
                let value: Expr = meta.require_list()?.parse_args()?;
                default = Some(value);
            } else if meta.path().is_ident("assignment") {
                let value: Punctuated<Ident, syn::Token![|]> = meta.require_list()?
                    .parse_args_with(Punctuated::<Ident, syn::Token![|]>::parse_separated_nonempty)?;

                assignments = value.into_iter().collect();
            }
        }

        Ok(Self { name, default, assignments })
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

pub fn derive_node_impl(input: DeriveInput) -> syn::Result<TokenStream> {
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
    let input_compilers = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let ident = input.ident;
            let ty = input.ty;

            quote! {
                #ident: <#ty as vislum_op::compile::CompileInput>::compile_input(ctx, node, #index)?
            }
        });

    let input_definitions = inputs.iter()
        .map(|input| {
            let ty = input.ty;
            let name = input.attrs.name.clone()
                .map(|name| name.value())
                .unwrap_or_else(|| input.ident.to_string());

            let assignments = if input.attrs.assignments.is_empty() {
                quote! { vislum_op::node_type::AssignmentTypes::ALL }
            } else {
                let assignments = input.attrs.assignments
                    .iter()
                    .map(|assignment| quote! { vislum_op::node_type::AssignmentTypes::#assignment });

                quote! { #(#assignments)|* }
            };

            quote! {
                <#ty as vislum_op::compile::GetInputDefinition>::get_input_definition(
                    #name,
                    #assignments,
                )
            }
        });

    let output_definition = outputs.iter()
        .map(|output| {
            let ty = output.ty;
            let name = output.attrs.name.clone()
                .map(|name| name.value())
                .unwrap_or_else(|| output.ident.to_string());

            quote! {
                <#ty as vislum_op::compile::GetOutputDefinition>::get_output_definition(#name)
            }
        });

    let outputs_len = outputs.len();
    let output_indexes = (0..outputs_len).map(|index| quote! { #index });
    let output_idents = outputs.iter().map(|output| output.ident).collect::<Vec<_>>();
    let state_idents = states.iter().map(|state| state.ident);

    let result = quote! {
        #[automatically_derived]
        impl vislum_op::compile::CompileNode for #ident {
            fn compile_node(ctx: &mut vislum_op::compile::CompilationContext, node_id: vislum_op::node::NodeId, node: &vislum_op::node::NodeBlueprint) -> Result<vislum_op::eval::NodeRef, ()> {
                Ok(vislum_op::eval::NodeRef::new(node_id, Self {
                    #(#input_compilers,)*
                    #(#output_idents: Default::default(),)*
                    #(#state_idents: Default::default(),)*
                }))
            }
        }

        #[automatically_derived]
        impl vislum_op::eval::GetOutput for #ident {
            fn get_output(&self, output_id: vislum_op::node::OutputId) -> Option<TaggedValue> {
                match output_id {
                    #(#output_indexes => vislum_op::eval::GetOutputValue::get_output_value(&self.#output_idents),)*
                    _ => None,
                }
            }
        }

        #[automatically_derived]
        impl vislum_op::node_type::RegisterNodeType for #ident {
            fn register_node_type(registry: &mut vislum_op::node_type::NodeTypeRegistry) {
                registry.add(vislum_op::node_type::NodeType::new(
                    vislum_op::node_type::NodeTypeId::new(#operator_type_id),
                    vec![
                        #(#input_definitions,)*
                    ],
                    vec![
                        #(#output_definition,)*
                    ],
                    <#ident as vislum_op::compile::CompileNode>::compile_node,
                ));
            }
        }

        /// Blanked implementaton.
        /// 
        /// Ensures the [`Eval`] trait has been implemented.
        #[automatically_derived]
        impl vislum_op::eval::Node for #ident {}
    };

    Ok(result)
}
