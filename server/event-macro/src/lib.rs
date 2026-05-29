use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Expr, Fields, Lit, Meta};

#[proc_macro_derive(EventListenerDispatcher, attributes(events, event_listener))]
pub fn derive_event_listener_dispatcher(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_event_listener_dispatcher(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_event_listener_dispatcher(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse #[events(Event1, Event2, ...)] attribute (validated but not used in codegen)
    let _events = parse_events_attribute(&input.attrs)?;

    // Parse fields with #[event_listener(source = "...")] annotations
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(input, "EventListenerDispatcher requires named fields")),
        },
        _ => return Err(syn::Error::new_spanned(input, "EventListenerDispatcher can only be derived on structs")),
    };

    let mut source_fields: Vec<proc_macro2::TokenStream> = Vec::new();
    for field in fields {
        let _field_name = field.ident.as_ref().unwrap();
        for attr in &field.attrs {
            if attr.path().is_ident("event_listener") {
                let source_expr = parse_event_listener_attribute(attr)?;
                source_fields.push(quote! {
                    {
                        let ids: Vec<_> = #source_expr.collect();
                        for id in &ids {
                            crate::game::event::EventListener::<E>::on_event(id, self, event, fold);
                        }
                    }
                });
            }
        }
    }

    // Generate a single generic dispatch method
    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            /// Dispatch an event to all registered listener identifiers.
            /// The event type is inferred from the call site.
            pub fn dispatch_event<E>(&mut self, event: &E, fold: &mut <E as crate::game::event::EventData>::FoldValue)
            where
                E: crate::game::event::EventData,
            {
                #(#source_fields)*
            }
        }
    };

    Ok(expanded)
}

fn parse_events_attribute(attrs: &[Attribute]) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    for attr in attrs {
        if attr.path().is_ident("events") {
            let meta = &attr.meta;
            return match meta {
                Meta::List(meta_list) => {
                    let mut events = Vec::new();
                    meta_list.parse_nested_meta(|meta| {
                        let ident = meta.path.require_ident()?;
                        events.push(quote! { #ident });
                        Ok(())
                    })?;
                    Ok(events)
                }
                _ => Err(syn::Error::new_spanned(
                    attr,
                    "expected #[events(Event1, Event2, ...)]",
                )),
            };
        }
    }
    // No events attribute is fine — just no events to dispatch
    Ok(Vec::new())
}

fn parse_event_listener_attribute(attr: &Attribute) -> syn::Result<Expr> {
    let mut source_expr = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("source") {
            let value = meta.value()?;
            let lit: Lit = value.parse()?;
            if let Lit::Str(s) = lit {
                let expr: Expr = syn::parse_str(&s.value())?;
                source_expr = Some(expr);
                Ok(())
            } else {
                Err(syn::Error::new_spanned(lit, "expected string literal"))
            }
        } else {
            Err(syn::Error::new_spanned(&meta.path, "unknown attribute"))
        }
    })?;

    source_expr.ok_or_else(|| syn::Error::new_spanned(attr, "expected #[event_listener(source = \"...\")]"))
}