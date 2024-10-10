use darling::{
    ast::{Data, Fields, NestedMeta, Style},
    util::Ignored,
    FromDeriveInput, FromMeta,
};
use manyhow::manyhow;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, DeriveInput, Expr, ExprArray, Field, Generics, Ident, Path, Type, Visibility,
};

macro_rules! combine_expr_arrays {
    ($($arr:ident),+) => {
        $(let $arr = combine_expr_array(&$arr);)+
    };
}

#[manyhow]
#[proc_macro_derive(Module, attributes(module))]
pub fn module(input: TokenStream) -> manyhow::Result<TokenStream> {
    let app_mod_path: Path = parse_quote!(crate::app);
    let app_msg_path: Path = parse_quote!(#app_mod_path::AppMsg);
    let mod_path: Path = parse_quote!(crate::module::new);
    let module_path: Path = parse_quote!(#mod_path::Module);
    let module_registry_path: Path = parse_quote!(#mod_path::ModuleRegistry);
    let module_event_path: Path = parse_quote!(#mod_path::ModuleEvent);
    let module_config_path: Path = parse_quote!(#mod_path::ModuleConfig);
    let module_widget_path: Path = parse_quote!(#mod_path::ModuleWidget);

    let derive_input: DeriveInput = syn::parse2(input)?;
    let ModuleStruct {
        vis,
        ident,
        generics,

        widget,
        type_fields,
        methods:
            ModuleMethods {
                new,
                init,
                cycle,
                widget_state,
            },
    } = ModuleStruct::from_derive_input(&derive_input)?;

    combine_expr_arrays![new, init, cycle, widget_state];

    let (impl_gen, ty_gen, where_gen) = generics.split_for_impl();

    let (
        (config_path, config_ty),
        (init_input_path, init_input_ty),
        (init_output_path, init_output_ty),
        (cycle_input_path, cycle_input_ty),
        (event_path, event_ty),
    ) = match &type_fields {
        Some(ModuleTypeFields {
            config,
            init_input,
            init_output,
            cycle_input,
            event,
        }) => {
            let (event_path, event_ty) = event_info(&vis, &ident, event, &module_event_path);

            let debug_path: Path = parse_quote!(Debug);
            let serialize_path: Path = parse_quote!(serde::Serialize);
            let deserialize_path: Path = parse_quote!(serde::Deserialize);

            let config = struct_info_or_empty(
                &vis,
                &ident,
                "Config",
                config,
                Some(vec![
                    debug_path,
                    parse_quote!(smart_default::SmartDefault),
                    serialize_path,
                    deserialize_path,
                ]),
                Some(|path| {
                    quote! {
                        impl #module_config_path for #path {}
                    }
                }),
            );
            let init_input = struct_info_or_empty::<AdditionalFn>(
                &vis,
                &ident,
                "InitInput",
                init_input,
                None,
                None,
            );
            let init_output = struct_info_or_empty::<AdditionalFn>(
                &vis,
                &ident,
                "InitOutput",
                init_output,
                None,
                None,
            );

            let cycle_input = struct_info_or_empty::<AdditionalFn>(
                &vis,
                &ident,
                "CycleInput",
                cycle_input,
                None,
                None,
            );

            (
                config,
                init_input,
                init_output,
                cycle_input,
                (event_path, Some(event_ty)),
            )
        }
        None => (
            empty_module_ty_info(),
            empty_module_ty_info(),
            empty_module_ty_info(),
            empty_module_ty_info(),
            empty_module_ty_info(),
        ),
    };

    Ok(quote! {
        impl #impl_gen #module_path for #ident #ty_gen #where_gen {
            type Config = #config_path;
            type InitInput = #init_input_path;
            type InitOutput = #init_output_path;
            type CycleInput = #cycle_input_path;
            type Event = #event_path;
            type Widget = #widget;

            fn new(config: Self::Config) -> miette::Result<Self>
            where
                Self: Sized
            {
                Ok(#new)
            }

            async fn init(
                &mut self,
                init_input: Self::InitInput
            ) -> miette::Result<Self::InitOutput> {
                Ok(#init)
            }

            async fn cycle(
                &mut self,
                registry: &mut #module_registry_path,
                cycle_input: Self::CycleInput,
                event: Self::Event
            ) -> miette::Result<Option<#app_msg_path>> {
                Ok(#cycle)
            }

            fn widget_state(
                &self,
                config: <Self::Widget as #module_widget_path<Self>>::Config,
            ) -> <Self::Widget as #module_widget_path<Self>>::State {
                #widget_state
            }
        }

        #config_ty
        #init_input_ty
        #init_output_ty
        #cycle_input_ty
        #event_ty
    })
}

fn empty_module_ty_info() -> (Type, Option<TokenStream>) {
    (parse_quote!(()), None)
}

type AdditionalFn = fn(Type) -> TokenStream;

fn struct_info_or_empty<F>(
    vis: &Visibility,
    main: &Ident,
    suffix: &str,
    fields: &[StructField],
    derives: Option<Vec<Path>>,
    additional: Option<F>,
) -> (Type, Option<TokenStream>)
where
    F: Fn(Type) -> TokenStream,
{
    match fields.is_empty() {
        true => empty_module_ty_info(),
        false => {
            let (path, ty) = struct_info(vis, main, suffix, fields, derives, additional);
            (path, Some(ty))
        }
    }
}

fn struct_info(
    vis: &Visibility,
    main: &Ident,
    suffix: &str,
    fields: &[StructField],
    derives: Option<Vec<Path>>,
    additional: Option<impl Fn(Type) -> TokenStream>,
) -> (Type, TokenStream) {
    let name = format_ident!("{main}{suffix}");
    let path: Type = parse_quote!(#name);
    let ty_fields = struct_fields_with_attrs(fields);
    let derives = derives.map(|derives| quote!(#[derive(#(#derives),*)]));
    let additional = additional.map(|additional| additional(path.clone()));
    let ty = quote! {
        #derives
        #vis struct #name {
            #(#ty_fields),*
        }

        #additional
    };

    (path, ty)
}

fn struct_fields_with_attrs(fields: &[StructField]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|StructField { name, ty, attrs }| {
            let attrs = match attrs {
                Some(attrs) => {
                    let ModuleTypeFieldAttrs { default } = attrs;
                    let default = quote!(#[default = #default]);

                    Some(quote! {
                        #default
                    })
                }
                None => None,
            };

            quote! {
                #attrs
                #name: #ty
            }
        })
        .collect()
}

fn event_info(
    vis: &Visibility,
    main: &Ident,
    variants: &[EventVariant],
    event_path: &Path,
) -> (Type, TokenStream) {
    let name = format_ident!("{main}Event");
    let path: Type = parse_quote!(#name);
    let variants = variants.iter().map(|EventVariant { name, field }| {
        let fields = match field.len() {
            0 => None,
            _ => Some(match field.len() > 1 {
                true => {
                    let fields = struct_fields_with_attrs(field);
                    quote!({#(#fields),*})
                }
                false => {
                    let field = field.first().as_ref().map(|StructField { ty, .. }| ty);
                    quote!((#field))
                }
            }),
        };

        quote!(#name #fields)
    });
    let ty = quote! {
        #[derive(Debug, Clone)]
        #vis enum #name {
            #(#variants),*
        }

        impl #event_path for #name {}
    };

    (path, ty)
}

fn combine_expr_array(arr: &ExprArray) -> TokenStream {
    let exprs = arr.elems.iter();

    quote! {
        #(#exprs);*
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(module), forward_attrs, supports(struct_any))]
struct ModuleStruct {
    vis: Visibility,
    ident: Ident,
    generics: Generics,

    widget: Path,
    type_fields: Option<ModuleTypeFields>,
    methods: ModuleMethods,
}

#[derive(FromMeta)]
struct ModuleTypeFields {
    #[darling(default, multiple)]
    config: Vec<StructField>,
    #[darling(default, multiple)]
    init_input: Vec<StructField>,
    #[darling(default, multiple)]
    init_output: Vec<StructField>,
    #[darling(default, multiple)]
    cycle_input: Vec<StructField>,
    #[darling(multiple)]
    event: Vec<EventVariant>,
}

#[derive(FromMeta)]
struct StructField {
    name: Ident,
    ty: Type,
    attrs: Option<ModuleTypeFieldAttrs>,
}

#[derive(FromMeta)]
struct ModuleTypeFieldAttrs {
    default: Option<Expr>,
}

#[derive(FromMeta)]
struct EventVariant {
    name: Ident,
    #[darling(default, multiple)]
    field: Vec<StructField>,
}

#[derive(FromMeta)]
struct ModuleMethods {
    new: ExprArray,
    init: ExprArray,
    cycle: ExprArray,
    widget_state: ExprArray,
}

#[manyhow]
#[proc_macro_attribute]
pub fn module_widget(attr: TokenStream, input: TokenStream) -> manyhow::Result<TokenStream> {
    let mod_path: Path = parse_quote!(crate::module::new);
    let module_widget_path: Path = parse_quote!(#mod_path::ModuleWidget);
    let module_widget_state_path: Path = parse_quote!(#mod_path::ModuleWidgetState);
    let module_widget_config_path: Path = parse_quote!(#mod_path::ModuleWidgetConfig);
    let module_widget_style_path: Path = parse_quote!(#mod_path::ModuleWidgetStyle);
    let module_widget_event_path: Path = parse_quote!(#mod_path::ModuleWidgetEvent);
    let module_widget_update_output_path: Path = parse_quote!(#mod_path::ModuleWidgetUpdateOutput);

    let attr_args = NestedMeta::parse_meta_list(attr)?;
    let ModuleWidgetAttr {
        module,
        type_fields,
        methods: ModuleWidgetMethods { view, update },
    } = ModuleWidgetAttr::from_list(&attr_args)?;
    combine_expr_arrays![view, update];

    let derive_input: DeriveInput = syn::parse2(input)?;
    let ModuleWidget {
        vis,
        ident,
        data,
        generics,
    } = ModuleWidget::from_derive_input(&derive_input)?;
    let attrs = &derive_input.attrs;
    let (impl_gen, ty_gen, where_gen) = generics.split_for_impl();

    let state_name = format_ident!("{ident}State");
    let Fields {
        fields: state_fields,
        style,
        ..
    } = data.take_struct().unwrap();
    let state_fields = match style {
        Style::Tuple => quote!((#(#state_fields),*);),
        Style::Struct => quote!({#(#state_fields),*}),
        Style::Unit => quote!(;),
    };

    let ((config_path, config_ty), (style_path, style_ty), (event_path, event_ty)) =
        match &type_fields {
            Some(ModuleWidgetTypeFields {
                config,
                style,
                event,
            }) => {
                let debug_path: Path = parse_quote!(Debug);
                let serialize_path: Path = parse_quote!(serde::Serialize);
                let deserialize_path: Path = parse_quote!(serde::Deserialize);

                let (style_path, style_ty) = struct_info_or_empty(
                    &vis,
                    &ident,
                    "Style",
                    style,
                    Some(vec![
                        debug_path.clone(),
                        serialize_path.clone(),
                        deserialize_path.clone(),
                    ]),
                    Some(|path| {
                        quote! {
                            impl #module_widget_style_path for #path {}
                        }
                    }),
                );
                let config = struct_info_or_empty(
                    &vis,
                    &ident,
                    "Config",
                    config,
                    Some(vec![
                        debug_path,
                        parse_quote!(smart_default::SmartDefault),
                        serialize_path,
                        deserialize_path,
                    ]),
                    Some(|path| {
                        quote! {
                            impl #module_widget_config_path for #path {
                                type Style = #style_path;
                            }
                        }
                    }),
                );
                let (event_path, event_ty) =
                    event_info(&vis, &ident, event, &module_widget_event_path);

                (config, (style_path, style_ty), (event_path, Some(event_ty)))
            }
            None => (
                empty_module_ty_info(),
                empty_module_ty_info(),
                empty_module_ty_info(),
            ),
        };

    Ok(quote! {
        #[derive(Debug)]
        #vis struct #ident;

        #(#attrs)*
        #vis struct #state_name #ty_gen #where_gen #state_fields

        impl #impl_gen #module_widget_state_path for #state_name #ty_gen #where_gen {}

        impl #impl_gen #module_widget_path<#module> for #ident #where_gen {
            type Config = #config_path;
            type State = #state_name #ty_gen;
            type Event = #event_path;

            fn view<'a>(
                self,
                style: Option<std::sync::Arc<#style_path>>,
                state: std::sync::Arc<tokio::sync::Mutex<Self::State>>
            ) -> iced::Element<'a, Self::Event, iced::Theme, iced::Renderer> {
                use std::ops::Deref;
                let lock_state = state.blocking_lock();
                let state = lock_state.deref();
                #view
            }

            fn update(
                self,
                state: std::sync::Arc<tokio::sync::Mutex<Self::State>>,
                event: Self::Event
            ) -> Option<#module_widget_update_output_path<#module>> {
                use std::ops::DerefMut;
                let mut lock_state = state.blocking_lock();
                let state = lock_state.deref_mut();
                #update
            }
        }

        #config_ty
        #style_ty
        #event_ty
    })
}

#[derive(FromMeta)]
struct ModuleWidgetAttr {
    module: Path,
    #[darling(default)]
    type_fields: Option<ModuleWidgetTypeFields>,
    methods: ModuleWidgetMethods,
}

#[derive(Default, FromMeta)]
struct ModuleWidgetTypeFields {
    #[darling(default, multiple)]
    config: Vec<StructField>,
    #[darling(default, multiple)]
    style: Vec<StructField>,
    #[darling(default, multiple)]
    event: Vec<EventVariant>,
}

#[derive(FromMeta)]
struct ModuleWidgetMethods {
    view: ExprArray,
    update: ExprArray,
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_any))]
struct ModuleWidget {
    vis: Visibility,
    ident: Ident,
    data: Data<Ignored, Field>,
    generics: Generics,
}
