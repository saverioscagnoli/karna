use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type, parse_macro_input};

#[proc_macro_derive(Getters, attributes(get))]
pub fn getters_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let getters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| {
                    let attr = field
                        .attrs
                        .iter()
                        .find(|attr| attr.path().is_ident("get"))?;
                    let field_name = field.ident.as_ref()?;
                    let field_type = &field.ty;

                    let mut cast_target: Option<Type> = None;
                    let mut func_name: Option<Ident> = None;
                    let mut style = "ref";

                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("cast") {
                            let value = meta.value()?;
                            let ty: Type = value.parse()?;
                            cast_target = Some(ty);
                            Ok(())
                        } else if meta.path.is_ident("fn") {
                            let value = meta.value()?;
                            let ident: Ident = value.parse()?;
                            func_name = Some(ident);
                            Ok(())
                        } else if meta.path.is_ident("copied") {
                            style = "copied";
                            Ok(())
                        } else if meta.path.is_ident("cloned") {
                            style = "cloned";
                            Ok(())
                        } else {
                            Ok(())
                        }
                    });

                    let base_expr = quote! { self.#field_name };

                    let processed_expr = if let Some(func) = func_name {
                        quote! { #base_expr.#func() }
                    } else {
                        base_expr
                    };

                    if let Some(target) = cast_target {
                        Some(quote! {
                            pub fn #field_name(&self) -> #target {
                                #processed_expr as #target
                            }
                        })
                    } else {
                        match style {
                            "copied" => Some(quote! {
                                pub fn #field_name(&self) -> #field_type {
                                    #processed_expr
                                }
                            }),
                            "cloned" => Some(quote! {
                                pub fn #field_name(&self) -> #field_type {
                                    #processed_expr.clone()
                                }
                            }),
                            _ => Some(quote! {
                                pub fn #field_name(&self) -> &#field_type {
                                    &#processed_expr
                                }
                            }),
                        }
                    }
                })
                .collect::<Vec<_>>(),
            _ => vec![],
        },
        _ => vec![],
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
