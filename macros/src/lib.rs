use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, MetaNameValue, Type, parse_macro_input};

#[proc_macro_derive(Get, attributes(get))]
pub fn derive_get(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Get only supports structs with named fields"),
        },
        _ => panic!("Get only supports structs"),
    };

    let mut getters = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        for attr in &field.attrs {
            if attr.path().is_ident("get") {
                let getter = parse_get_attribute(attr, field_name, field_type);
                getters.push(getter);
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Set, attributes(set))]
pub fn derive_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Set only supports structs with named fields"),
        },
        _ => panic!("Set only supports structs"),
    };

    let mut setters = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        for attr in &field.attrs {
            if attr.path().is_ident("set") {
                let setter = parse_set_attribute(attr, field_name, field_type);
                setters.push(setter);
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(With, attributes(with))]
pub fn derive_with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("With only supports structs with named fields"),
        },
        _ => panic!("With only supports structs"),
    };

    let mut withers = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        for attr in &field.attrs {
            if attr.path().is_ident("with") {
                let wither = parse_with_attribute(attr, field_name, field_type);
                withers.push(wither);
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#withers)*
        }
    };

    TokenStream::from(expanded)
}

struct GetConfig {
    prop: Option<syn::Ident>,
    pre: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    name: Option<syn::Ident>,
    copied: bool,
    dirty: bool,
}

fn parse_get_attribute(
    attr: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &Type,
) -> proc_macro2::TokenStream {
    let mut config = GetConfig {
        prop: None,
        pre: None,
        cast: None,
        ty: None,
        name: None,
        copied: false,
        dirty: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                Meta::Path(path) if path.is_ident("copied") => {
                    config.copied = true;
                }
                Meta::Path(path) if path.is_ident("dirty") => {
                    config.dirty = true;
                }
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("prop") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("pre") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.cast = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("ty") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.ty = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("name") {
                        if let syn::Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name =
                                    Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    } else if matches!(attr.meta, Meta::Path(_)) {
        // Just #[get] with no parameters
    }

    let method_name = config.name.clone().unwrap_or_else(|| {
        if let Some(prop) = &config.prop {
            syn::Ident::new(&format!("{}_{}", field_name, prop), field_name.span())
        } else {
            field_name.clone()
        }
    });

    let return_type = if let Some(ty) = &config.ty {
        // Explicit type annotation provided - use as-is
        quote! { #ty }
    } else if let Some(cast_type) = &config.cast {
        quote! { #cast_type }
    } else if config.copied {
        quote! { #field_type }
    } else {
        // Return reference - need to extract inner type if dirty
        if config.dirty && config.prop.is_none() {
            // Extract T from DirtyTracked<T> to return &T instead of &DirtyTracked<T>
            // We need to parse the field_type to get the inner type
            if let Type::Path(type_path) = field_type {
                if let Some(segment) = type_path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            quote! { &#inner_type }
                        } else {
                            quote! { &#field_type }
                        }
                    } else {
                        quote! { &#field_type }
                    }
                } else {
                    quote! { &#field_type }
                }
            } else {
                quote! { &#field_type }
            }
        } else {
            quote! { &#field_type }
        }
    };

    let body = if let Some(prop) = &config.prop {
        let field_access = if config.dirty {
            quote! { self.#field_name.value.#prop }
        } else {
            quote! { self.#field_name.#prop }
        };

        if let Some(pre) = &config.pre {
            if let Some(cast_type) = &config.cast {
                quote! { #field_access.#pre() as #cast_type }
            } else if config.copied {
                quote! { #field_access.#pre() }
            } else {
                quote! { &#field_access.#pre() }
            }
        } else if let Some(cast_type) = &config.cast {
            quote! { #field_access as #cast_type }
        } else if config.copied {
            quote! { #field_access }
        } else {
            quote! { &#field_access }
        }
    } else if let Some(pre) = &config.pre {
        let field_access = if config.dirty {
            quote! { self.#field_name.value }
        } else {
            quote! { self.#field_name }
        };

        if let Some(cast_type) = &config.cast {
            quote! { #field_access.#pre() as #cast_type }
        } else if config.copied {
            quote! { #field_access.#pre() }
        } else {
            quote! { &#field_access.#pre() }
        }
    } else {
        let field_access = if config.dirty {
            quote! { self.#field_name.value }
        } else {
            quote! { self.#field_name }
        };

        if let Some(cast_type) = &config.cast {
            quote! { #field_access as #cast_type }
        } else if config.copied {
            quote! { #field_access }
        } else {
            quote! { &#field_access }
        }
    };

    quote! {
        #[inline]
        pub fn #method_name(&self) -> #return_type {
            #body
        }
    }
}

struct SetConfig {
    prop: Option<syn::Ident>,
    pre: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    name: Option<syn::Ident>,
    into: bool,
    dirty: bool,
}

fn parse_set_attribute(
    attr: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &Type,
) -> proc_macro2::TokenStream {
    let mut config = SetConfig {
        prop: None,
        pre: None,
        cast: None,
        ty: None,
        name: None,
        into: false,
        dirty: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                Meta::Path(path) if path.is_ident("into") => {
                    config.into = true;
                }
                Meta::Path(path) if path.is_ident("dirty") => {
                    config.dirty = true;
                }
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("prop") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("pre") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.cast = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("ty") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.ty = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("name") {
                        if let syn::Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name =
                                    Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let method_name = config.name.clone().unwrap_or_else(|| {
        if let Some(prop) = &config.prop {
            syn::Ident::new(&format!("set_{}_{}", field_name, prop), field_name.span())
        } else {
            syn::Ident::new(&format!("set_{}", field_name), field_name.span())
        }
    });

    let param_type = if let Some(ty) = &config.ty {
        // Explicit type annotation provided
        if config.into {
            quote! { impl Into<#ty> }
        } else {
            quote! { #ty }
        }
    } else if config.into {
        quote! { impl Into<#field_type> }
    } else {
        quote! { #field_type }
    };

    let value_expr = if config.into {
        quote! { value.into() }
    } else {
        quote! { value }
    };

    let body = if let Some(prop) = &config.prop {
        if config.dirty {
            if let Some(cast_type) = &config.cast {
                quote! { self.#field_name.value.#prop = #value_expr as #cast_type; }
            } else {
                quote! { self.#field_name.value.#prop = #value_expr; }
            }
        } else {
            if let Some(cast_type) = &config.cast {
                quote! { self.#field_name.#prop = #value_expr as #cast_type; }
            } else {
                quote! { self.#field_name.#prop = #value_expr; }
            }
        }
    } else if let Some(cast_type) = &config.cast {
        if config.dirty {
            quote! { self.#field_name = DirtyTracked::new(#value_expr as #cast_type); }
        } else {
            quote! { self.#field_name = #value_expr as #cast_type; }
        }
    } else {
        if config.dirty {
            quote! { self.#field_name = DirtyTracked::new(#value_expr); }
        } else {
            quote! { self.#field_name = #value_expr; }
        }
    };

    quote! {
        #[inline]
        pub fn #method_name(&mut self, value: #param_type) {
            #body
        }
    }
}

struct WithConfig {
    prop: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    name: Option<syn::Ident>,
    into: bool,
    dirty: bool,
}

fn parse_with_attribute(
    attr: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &Type,
) -> proc_macro2::TokenStream {
    let mut config = WithConfig {
        prop: None,
        cast: None,
        ty: None,
        name: None,
        into: false,
        dirty: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                Meta::Path(path) if path.is_ident("into") => {
                    config.into = true;
                }
                Meta::Path(path) if path.is_ident("dirty") => {
                    config.dirty = true;
                }
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("prop") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.cast = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("ty") {
                        if let syn::Expr::Path(expr_path) = value {
                            config.ty = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("name") {
                        if let syn::Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name =
                                    Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let method_name = config.name.clone().unwrap_or_else(|| {
        if let Some(prop) = &config.prop {
            syn::Ident::new(&format!("with_{}_{}", field_name, prop), field_name.span())
        } else {
            syn::Ident::new(&format!("with_{}", field_name), field_name.span())
        }
    });

    let param_type = if let Some(ty) = &config.ty {
        // Explicit type annotation provided
        if config.into {
            quote! { impl Into<#ty> }
        } else {
            quote! { #ty }
        }
    } else if config.into {
        quote! { impl Into<#field_type> }
    } else {
        quote! { #field_type }
    };

    let value_expr = if config.into {
        quote! { value.into() }
    } else {
        quote! { value }
    };

    let body = if let Some(prop) = &config.prop {
        if config.dirty {
            if let Some(cast_type) = &config.cast {
                quote! {
                    self.#field_name.value.#prop = #value_expr as #cast_type;
                    self
                }
            } else {
                quote! {
                    self.#field_name.value.#prop = #value_expr;
                    self
                }
            }
        } else {
            if let Some(cast_type) = &config.cast {
                quote! {
                    self.#field_name.#prop = #value_expr as #cast_type;
                    self
                }
            } else {
                quote! {
                    self.#field_name.#prop = #value_expr;
                    self
                }
            }
        }
    } else if let Some(cast_type) = &config.cast {
        if config.dirty {
            quote! {
                self.#field_name = DirtyTracked::new(#value_expr as #cast_type);
                self
            }
        } else {
            quote! {
                self.#field_name = #value_expr as #cast_type;
                self
            }
        }
    } else {
        if config.dirty {
            quote! {
                self.#field_name = DirtyTracked::new(#value_expr);
                self
            }
        } else {
            quote! {
                self.#field_name = #value_expr;
                self
            }
        }
    };

    quote! {
        #[inline]
        pub fn #method_name(mut self, value: #param_type) -> Self {
            #body
        }
    }
}
