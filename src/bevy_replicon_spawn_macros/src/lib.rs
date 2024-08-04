extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Comma, PathSep},
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, GenericArgument, Ident, Meta,
    Path, PathArguments, PathSegment, Token, Type, TypePath,
};

struct EventModifierArg {
    pub(crate) name: Ident,
    pub(crate) value: Type,
}

impl Parse for EventModifierArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { name, value })
    }
}

struct EventModifierAttributes {
    component: Type,
    spawner: Type,
}

fn parse_attributes(attrs: &[Attribute]) -> EventModifierAttributes {
    for attr in attrs {
        match &attr.meta {
            Meta::List(list) => {
                for segment in &list.path.segments {
                    if segment.ident != "modifier" {
                        continue;
                    }
                }
            }
            _ => continue,
        }

        let mut component = None;
        let mut spawner = None;
        for arg in attr
            .parse_args_with(Punctuated::<EventModifierArg, syn::Token![,]>::parse_terminated)
            .expect("Failed to parse arguments")
        {
            match arg.name.to_string().as_str() {
                "component" => component = Some(arg.value),
                "spawner" => spawner = Some(arg.value),
                _ => panic!("Unknown argument `{}`", arg.name),
            }
        }

        if let (Some(component), Some(spawner)) = (component, spawner) {
            return EventModifierAttributes { component, spawner };
        }
    }
    panic!("Missing required attribute `modifier` with arguments `component`, `input`, `metadata`, `output`, `priority`")
}

fn remove_system_param_lifetimes(field: &Type) -> Type {
    match &field {
        Type::Path(path) => {
            if path.path.segments.len() != 1 {
                panic!("Unsupported field type");
            }
            let segment = &path.path.segments[0];
            let PathArguments::AngleBracketed(path_args) = &segment.arguments else {
                panic!("Unsupported field type");
            };
            match segment.ident.to_string().as_str() {
                "EventWriter" | "Res" | "ResMut" => {
                    let mut segments = Punctuated::<PathSegment, PathSep>::new();
                    let mut args = Punctuated::<GenericArgument, Comma>::new();
                    for arg in path_args.args.iter().skip(1) {
                        args.push(arg.clone());
                    }
                    segments.push(PathSegment {
                        ident: segment.ident.clone(),
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: path_args.lt_token,
                            args,
                            gt_token: path_args.gt_token,
                        }),
                    });
                    return Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments,
                        },
                    });
                }
                "Query" => {
                    let mut segments = Punctuated::<PathSegment, PathSep>::new();
                    let mut args = Punctuated::<GenericArgument, Comma>::new();
                    for arg in path_args.args.iter().skip(2) {
                        args.push(arg.clone());
                    }
                    segments.push(PathSegment {
                        ident: segment.ident.clone(),
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: path_args.lt_token,
                            args,
                            gt_token: path_args.gt_token,
                        }),
                    });
                    return Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments,
                        },
                    });
                }
                _ => panic!("Unsupported field type"),
            }
        }
        _ => panic!("Unsupported field type"),
    }
}

#[proc_macro_derive(SpawnContext, attributes(modifier))]
pub fn derive_spawn_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let attributes = parse_attributes(&ast.attrs);

    let (user_impl_generics, _, _) = ast.generics.split_for_impl();

    let struct_name = &ast.ident;

    let data = match ast.data {
        Data::Struct(data) => data,
        _ => panic!("Only structs are supported"),
    };

    let component_ty = attributes.component;
    let spawner_ty = attributes.spawner;

    let system_param_names = data.fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        quote! {
            #field_ident,
        }
    });

    let system_params = data.fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("Field must have an identifier");
        let field_ty = remove_system_param_lifetimes(&field.ty);
        quote! {
            #field_ident: #field_ty,
        }
    });

    let output = quote! {
        impl #user_impl_generics #struct_name #user_impl_generics {
            pub fn system(
                #(#system_params)*
                mut commands: Commands,
                p_query: Query<(Entity, &#component_ty), (Added<#component_ty>, Added<Replicated>)>,
            ) {
                let mut context = #struct_name {
                    #(#system_param_names)*
                };
                for (entity, event) in &p_query {
                    #spawner_ty(commands.entity(entity), &mut context, event);
                }
            }
        }

        impl #user_impl_generics bevy_replicon_spawn::prelude::SpawnContext for #struct_name #user_impl_generics {
            fn register_type(app: &mut App) -> &mut App {
                app.replicate::<#component_ty>();
                app.add_systems(Update, #struct_name ::system);
                app
            }
        }
    };

    TokenStream::from(output)
}
