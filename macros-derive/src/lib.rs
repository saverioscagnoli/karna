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
                .flat_map(|field| {
                    let attr = field
                        .attrs
                        .iter()
                        .find(|attr| attr.path().is_ident("get"))?;
                    let field_name = field.ident.as_ref()?;
                    let field_type = &field.ty;
                    let mut cast_target: Option<Type> = None;
                    let mut func_name: Option<Ident> = None;
                    let mut style = "ref";
                    let mut is_mut = false;
                    let mut suffixes = Vec::new();

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
                        } else if meta.path.is_ident("mut") {
                            is_mut = true;
                            Ok(())
                        } else if meta.path.is_ident("suffix") {
                            let value = meta.value()?;
                            let suffix: Ident = value.parse()?;
                            suffixes.push(suffix);
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

                    let mut result = Vec::new();

                    // Generate the immutable getter
                    if let Some(target) = cast_target {
                        result.push(quote! {
                            #[inline]
                            pub fn #field_name(&self) -> #target {
                                #processed_expr as #target
                            }
                        });
                    } else {
                        match style {
                            "copied" => result.push(quote! {
                                #[inline]
                                pub fn #field_name(&self) -> #field_type {
                                    #processed_expr
                                }
                            }),
                            "cloned" => result.push(quote! {
                                #[inline]
                                pub fn #field_name(&self) -> #field_type {
                                    #processed_expr.clone()
                                }
                            }),
                            _ => result.push(quote! {
                                #[inline]
                                pub fn #field_name(&self) -> &#field_type {
                                    &#processed_expr
                                }
                            }),
                        }
                    }

                    // Generate the mutable getter if requested
                    if is_mut {
                        let mut_name =
                            Ident::new(&format!("{}_mut", field_name), field_name.span());
                        result.push(quote! {
                            #[inline]
                            pub fn #mut_name(&mut self) -> &mut #field_type {
                                &mut self.#field_name
                            }
                        });
                    }

                    // Generate suffix getters
                    for suffix in suffixes {
                        let suffix_getter =
                            Ident::new(&format!("{}_{}", field_name, suffix), field_name.span());

                        result.push(quote! {
                            #[inline]
                            pub fn #suffix_getter(&self) -> f32 {
                                self.#field_name.#suffix
                            }
                        });
                    }

                    Some(result)
                })
                .flatten()
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

#[proc_macro_derive(Setters, attributes(set))]
pub fn setters_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let setters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .flat_map(|field| {
                    // Check for #[set] attribute
                    let attr = field.attrs.iter().find(|a| a.path().is_ident("set"))?;
                    let field_name = field.ident.as_ref()?;
                    let field_type = &field.ty;
                    let mut func_name: Option<Ident> = None;
                    let mut by_ref = false;
                    let mut suffixes = Vec::new();

                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("fn") {
                            let value = meta.value()?;
                            let ident: Ident = value.parse()?;
                            func_name = Some(ident);
                            Ok(())
                        } else if meta.path.is_ident("ref") {
                            by_ref = true;
                            Ok(())
                        } else if meta.path.is_ident("suffix") {
                            let value = meta.value()?;
                            let suffix: Ident = value.parse()?;
                            suffixes.push(suffix);
                            Ok(())
                        } else {
                            Ok(())
                        }
                    });

                    let mut result = Vec::new();

                    // Generate main setter
                    let setter_name = func_name.unwrap_or_else(|| {
                        Ident::new(&format!("set_{}", field_name), field_name.span())
                    });
                    let param_type = if by_ref {
                        quote! { &#field_type }
                    } else {
                        quote! { #field_type }
                    };
                    let value_expr = if by_ref {
                        quote! { *value }
                    } else {
                        quote! { value }
                    };

                    result.push(quote! {
                        #[inline]
                        pub fn #setter_name(&mut self, value: #param_type) {
                            self.#field_name = #value_expr;
                        }
                    });

                    // Generate suffix setters
                    for suffix in suffixes {
                        let suffix_setter = Ident::new(
                            &format!("set_{}_{}", field_name, suffix),
                            field_name.span(),
                        );

                        result.push(quote! {
                            #[inline]
                            pub fn #suffix_setter(&mut self, value: f32) {
                                self.#field_name.#suffix = value;
                            }
                        });
                    }

                    Some(result)
                })
                .flatten()
                .collect::<Vec<_>>(),
            _ => vec![],
        },
        _ => vec![],
    };
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(With, attributes(with))]
pub fn with_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let with_methods = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_with = field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("with"));

                    if !has_with {
                        return None;
                    }

                    let field_name = field.ident.as_ref()?;
                    let field_type = &field.ty;
                    let method_name = Ident::new(&format!("with_{}", field_name), field_name.span());

                    let mut into = false;
                    let mut suffixes = Vec::new();
                    let mut use_deref = false;

                    for attr in &field.attrs {
                        if attr.path().is_ident("with") {
                            let _ = attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("into") {
                                    into = true;
                                    Ok(())
                                } else if meta.path.is_ident("suffix") {
                                    let value = meta.value()?;
                                    let suffix: Ident = value.parse()?;
                                    suffixes.push(suffix);
                                    Ok(())
                                } else if meta.path.is_ident("deref") {
                                    use_deref = true;
                                    Ok(())
                                } else {
                                    Ok(())
                                }
                            });
                        }
                    }

                    let mut methods = Vec::new();

                    // Generate main with_ method
                    // Direct field access (default behavior)
                    if into {
                    methods.push(quote! {
                            #[inline]
                            pub fn #method_name<T: Into<#field_type>>(mut self, #field_name: T) -> Self {
                                self.#field_name = #field_name.into();
                                self
                            }
                        });
                    } else {
                        methods.push(quote! {
                            #[inline]
                            pub fn #method_name(mut self, #field_name: #field_type) -> Self {
                                self.#field_name = #field_name;
                                self
                            }
                        });
                    }

                    // Generate suffix methods
                    for suffix in suffixes {
                        let suffix_method = Ident::new(&format!("with_{}_{}", field_name, suffix), field_name.span());

                        methods.push(quote! {
                            #[inline]
                            pub fn #suffix_method(mut self, value: f32) -> Self {
                                self.#field_name.#suffix = value;
                                self
                            }
                        });
                    }

                    Some(methods)
                })
                .flatten()
                .collect::<Vec<_>>(),
            _ => vec![],
        },
        _ => vec![],
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#with_methods)*
        }
    };

    TokenStream::from(expanded)
}
