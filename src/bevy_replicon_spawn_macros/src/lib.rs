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
    input: Type,
    metadata: Type,
    output: Type,
    priority: Type,
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
        let mut input = None;
        let mut metadata = None;
        let mut output = None;
        let mut priority = None;
        for arg in attr
            .parse_args_with(
                Punctuated::<EventModifierArg, syn::Token![,]>::parse_separated_nonempty,
            )
            .expect("Failed to parse arguments")
        {
            match arg.name.to_string().as_str() {
                "component" => component = Some(arg.value),
                "input" => input = Some(arg.value),
                "metadata" => metadata = Some(arg.value),
                "output" => output = Some(arg.value),
                "priority" => priority = Some(arg.value),
                _ => panic!("Unknown argument `{}`", arg.name),
            }
        }
        if let (Some(component), Some(input), Some(metadata), Some(output), Some(priority)) =
            (component, input, metadata, output, priority)
        {
            return EventModifierAttributes {
                component,
                input,
                metadata,
                output,
                priority,
            };
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
                            lt_token: path_args.lt_token.clone(),
                            args,
                            gt_token: path_args.gt_token.clone(),
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
                            lt_token: path_args.lt_token.clone(),
                            args,
                            gt_token: path_args.gt_token.clone(),
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

#[proc_macro_derive(ClientSpawnEvent, attributes(modifier))]
pub fn derive_client_spawn_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let attributes = parse_attributes(&ast.attrs);

    let (user_impl_generics, _, _) = ast.generics.split_for_impl();

    let struct_name = &ast.ident;

    let data = match ast.data {
        Data::Struct(data) => data,
        _ => panic!("Only structs are supported"),
    };

    let component_ty = attributes.component;
    let input_ty = attributes.input;
    let priority_ty = attributes.priority;
    let metadata_ty = attributes.metadata;
    let output_ty = attributes.output;

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
        #[derive(Component)]
        pub struct #component_ty {
            pub priority: #priority_ty,
            pub modify: fn(&mut #struct_name, &mut #metadata_ty, &mut #output_ty),
        }

        impl Ord for #component_ty {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.priority.cmp(&other.priority)
            }
        }

        impl PartialOrd for #component_ty {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.priority.partial_cmp(&other.priority)
            }
        }

        impl Eq for #component_ty {

        }

        impl PartialEq for #component_ty {
            fn eq(&self, other: &Self) -> bool {
                self.priority == other.priority
            }
        }

        impl #user_impl_generics #struct_name #user_impl_generics {
            pub fn system(
                #(#system_params)*
                mut p_events_in: EventReader<#input_ty>,
                p_modifiers: Query<&#component_ty>,
                mut p_events_out: EventWriter<#output_ty>,
            ) {
                let mut context = #struct_name {
                    #(#system_param_names)*
                };
                let modifiers = p_modifiers
                    .iter()
                    .sort::<&#component_ty>()
                    .collect::<Vec<_>>();
                for event in p_events_in.read() {
                    let Some(mut event_out) = #output_ty ::init(&mut context, event) else {
                        continue;
                    };
                    let mut metadata = #metadata_ty ::init(&mut context, event);
                    for modifier in &modifiers {
                        (modifier.modify)(&mut context, &mut metadata, &mut event_out);
                    }
                    p_events_out.send(event_out);
                }
            }
        }

        impl #user_impl_generics bevy_event_modifiers::prelude::EventModifierContext for #struct_name #user_impl_generics {
            fn register_type(app: &mut App) -> &mut App {
                app.add_event::<#input_ty>();
                app.add_event::<#output_ty>();
                app.add_systems(Update, #struct_name ::system.run_if(on_event::<#input_ty>()));
                app
            }
        }
    };

    TokenStream::from(output)
}

#[proc_macro_derive(ServerSpawnEvent, attributes(modifier))]
pub fn derive_server_spawn_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let attributes = parse_attributes(&ast.attrs);

    let (user_impl_generics, _, _) = ast.generics.split_for_impl();

    let struct_name = &ast.ident;

    let data = match ast.data {
        Data::Struct(data) => data,
        _ => panic!("Only structs are supported"),
    };

    let component_ty = attributes.component;
    let input_ty = attributes.input;
    let priority_ty = attributes.priority;
    let metadata_ty = attributes.metadata;
    let output_ty = attributes.output;

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

    let output = quote! {};

    TokenStream::from(output)
}
