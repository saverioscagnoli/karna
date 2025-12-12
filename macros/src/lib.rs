use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Fields, Lit, Meta, MetaNameValue, Token, Type, Visibility,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
};

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
    vis: Option<Visibility>,
    copied: bool,
    mutable: bool,
}

enum GetMeta {
    Mut,
    Meta(Meta),
}

impl Parse for GetMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![mut]) {
            let _ = input.parse::<Token![mut]>()?;
            Ok(GetMeta::Mut)
        } else {
            let meta = input.parse()?;
            Ok(GetMeta::Meta(meta))
        }
    }
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
        vis: None,
        copied: false,
        mutable: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(Punctuated::<GetMeta, Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                GetMeta::Mut => {
                    config.mutable = true;
                }
                GetMeta::Meta(m) => match m {
                    Meta::Path(path) if path.is_ident("copied") => {
                        config.copied = true;
                    }
                    Meta::NameValue(MetaNameValue { path, value, .. }) => {
                        if path.is_ident("prop") {
                            if let Expr::Path(expr_path) = value {
                                config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                            }
                        } else if path.is_ident("pre") {
                            if let Expr::Path(expr_path) = value {
                                config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                            }
                        } else if path.is_ident("cast") {
                            if let Expr::Path(expr_path) = value {
                                config.cast = Some(Type::Path(syn::TypePath {
                                    qself: None,
                                    path: expr_path.path.clone(),
                                }));
                            }
                        } else if path.is_ident("ty") {
                            if let Expr::Path(expr_path) = value {
                                config.ty = Some(Type::Path(syn::TypePath {
                                    qself: None,
                                    path: expr_path.path.clone(),
                                }));
                            }
                        } else if path.is_ident("name") {
                            if let Expr::Lit(expr_lit) = value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    config.name =
                                        Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                                }
                            }
                        } else if path.is_ident("visibility") {
                            if let Expr::Lit(expr_lit) = value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    config.vis = syn::parse_str(&lit_str.value()).ok();
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    let method_name = config.name.clone().unwrap_or_else(|| {
        let base_name_str = if let Some(prop) = &config.prop {
            format!("{}_{}", field_name, prop)
        } else {
            field_name.to_string()
        };

        let final_name_str = if config.mutable {
            format!("{}_mut", base_name_str)
        } else {
            base_name_str
        };

        syn::Ident::new(&final_name_str, field_name.span())
    });

    let vis = config.vis.unwrap_or_else(|| syn::parse_quote! { pub });

    let return_type = if let Some(ty) = &config.ty {
        quote! { #ty }
    } else if let Some(cast_type) = &config.cast {
        quote! { #cast_type }
    } else if config.mutable {
        quote! { &mut #field_type }
    } else if config.copied {
        quote! { #field_type }
    } else {
        quote! { &#field_type }
    };

    let self_arg = if config.mutable {
        quote! { &mut self }
    } else {
        quote! { &self }
    };

    let body = if let Some(prop) = &config.prop {
        let field_access = quote! { self.#field_name.#prop };

        if let Some(pre) = &config.pre {
            if let Some(cast_type) = &config.cast {
                quote! { #field_access.#pre() as #cast_type }
            } else if config.mutable {
                quote! { &mut #field_access.#pre() }
            } else if config.copied {
                quote! { #field_access.#pre() }
            } else {
                quote! { &#field_access.#pre() }
            }
        } else if let Some(cast_type) = &config.cast {
            quote! { #field_access as #cast_type }
        } else if config.mutable {
            quote! { &mut #field_access }
        } else if config.copied {
            quote! { #field_access }
        } else {
            quote! { &#field_access }
        }
    } else if let Some(pre) = &config.pre {
        let field_access = quote! { self.#field_name };

        if let Some(cast_type) = &config.cast {
            quote! { #field_access.#pre() as #cast_type }
        } else if config.mutable {
            quote! { &mut #field_access.#pre() }
        } else if config.copied {
            quote! { #field_access.#pre() }
        } else {
            quote! { &#field_access.#pre() }
        }
    } else {
        let field_access = quote! { self.#field_name };

        if let Some(cast_type) = &config.cast {
            quote! { #field_access as #cast_type }
        } else if config.mutable {
            quote! { &mut #field_access }
        } else if config.copied {
            quote! { #field_access }
        } else {
            quote! { &#field_access }
        }
    };

    quote! {
        #[inline]
        #vis fn #method_name(#self_arg) -> #return_type {
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
    vis: Option<Visibility>,
    also: Option<Expr>,
    into: bool,
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
        vis: None,
        also: None,
        into: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                Meta::Path(path) if path.is_ident("into") => {
                    config.into = true;
                }
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("prop") {
                        if let Expr::Path(expr_path) = value {
                            config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("pre") {
                        if let Expr::Path(expr_path) = value {
                            config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        if let Expr::Path(expr_path) = value {
                            config.cast = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("ty") {
                        if let Expr::Path(expr_path) = value {
                            config.ty = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("name") {
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name =
                                    Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                            }
                        }
                    } else if path.is_ident("visibility") {
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.vis = syn::parse_str(&lit_str.value()).ok();
                            }
                        }
                    } else if path.is_ident("also") {
                        config.also = Some(value);
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

    let vis = config.vis.unwrap_or_else(|| syn::parse_quote! { pub });

    let param_type = if let Some(ty) = &config.ty {
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

    let also_stmt = if let Some(also) = config.also {
        quote! { #also; }
    } else {
        quote! {}
    };

    let body = if let Some(prop) = &config.prop {
        if let Some(cast_type) = &config.cast {
            quote! { self.#field_name.#prop = #value_expr as #cast_type; }
        } else {
            quote! { self.#field_name.#prop = #value_expr; }
        }
    } else if let Some(cast_type) = &config.cast {
        quote! { self.#field_name = #value_expr as #cast_type; }
    } else {
        quote! { self.#field_name = #value_expr; }
    };

    quote! {
        #[inline]
        #vis fn #method_name(&mut self, value: #param_type) {
            #also_stmt
            #body
        }
    }
}

struct WithConfig {
    prop: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    name: Option<syn::Ident>,
    vis: Option<Visibility>,
    also: Option<Expr>,
    into: bool,
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
        vis: None,
        also: None,
        into: false,
    };

    if let Meta::List(meta_list) = &attr.meta {
        let nested = meta_list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap();

        for meta in nested {
            match meta {
                Meta::Path(path) if path.is_ident("into") => {
                    config.into = true;
                }
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("prop") {
                        if let Expr::Path(expr_path) = value {
                            config.prop = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        if let Expr::Path(expr_path) = value {
                            config.cast = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("ty") {
                        if let Expr::Path(expr_path) = value {
                            config.ty = Some(Type::Path(syn::TypePath {
                                qself: None,
                                path: expr_path.path.clone(),
                            }));
                        }
                    } else if path.is_ident("name") {
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name =
                                    Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                            }
                        }
                    } else if path.is_ident("visibility") {
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.vis = syn::parse_str(&lit_str.value()).ok();
                            }
                        }
                    } else if path.is_ident("also") {
                        config.also = Some(value);
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

    let vis = config.vis.unwrap_or_else(|| syn::parse_quote! { pub });

    let param_type = if let Some(ty) = &config.ty {
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

    let also_stmt = if let Some(also) = config.also {
        quote! { #also; }
    } else {
        quote! {}
    };

    let body = if let Some(prop) = &config.prop {
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
    } else if let Some(cast_type) = &config.cast {
        quote! {
            self.#field_name = #value_expr as #cast_type;
            self
        }
    } else {
        quote! {
            self.#field_name = #value_expr;
            self
        }
    };

    quote! {
        #[inline]
        #vis fn #method_name(mut self, value: #param_type) -> Self {
            #also_stmt
            #body
        }
    }
}

#[proc_macro_attribute]
pub fn dirty(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("dirty only supports structs with named fields"),
        },
        _ => panic!("dirty only supports structs"),
    };

    let mut new_fields = Vec::new();
    let mut dirty_fields = Vec::new();
    let mut bit_index = 0usize;

    // Process fields
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_vis = &field.vis;

        let mut is_dirty = false;
        let mut use_into = false;

        // Check for #[dirty] or #[dirty(into)] attribute
        for attr in &field.attrs {
            if attr.path().is_ident("dirty") {
                is_dirty = true;

                // Check if it has (into) parameter
                if let Meta::List(meta_list) = &attr.meta {
                    if let Ok(ident) = syn::parse2::<syn::Ident>(meta_list.tokens.clone()) {
                        if ident == "into" {
                            use_into = true;
                        }
                    }
                }
            }
        }

        // Keep the field without the dirty attribute
        let mut new_field = field.clone();
        new_field
            .attrs
            .retain(|attr| !attr.path().is_ident("dirty"));
        new_fields.push(new_field);

        if is_dirty {
            dirty_fields.push((
                field_name.clone(),
                field_type.clone(),
                bit_index,
                use_into,
                field_vis.clone(),
            ));
            bit_index += 1;
        }
    }

    // Add __tracker field
    new_fields.push(
        syn::Field::parse_named
            .parse2(quote! {
                __tracker: u8
            })
            .unwrap(),
    );

    // Generate const functions for bit masks
    let bit_consts: Vec<_> = dirty_fields
        .iter()
        .map(|(field_name, _, index, _, _)| {
            let const_name = field_name;
            quote! {
                #[inline]
                pub const fn #const_name() -> u8 {
                    1 << #index
                }
            }
        })
        .collect();

    // Generate setter methods
    let setters: Vec<_> = dirty_fields
        .iter()
        .map(|(field_name, field_type, _, use_into, field_vis)| {
            let setter_name = syn::Ident::new(&format!("set_{}", field_name), field_name.span());

            if *use_into {
                quote! {
                    #[inline]
                    #field_vis fn #setter_name(&mut self, value: impl Into<#field_type>) {
                        let value = value.into();
                        if self.#field_name != value {
                            self.__tracker |= Self::#field_name();
                            self.#field_name = value;
                        }
                    }
                }
            } else {
                quote! {
                    #[inline]
                    #field_vis fn #setter_name(&mut self, value: #field_type) {
                        if self.#field_name != value {
                            self.__tracker |= Self::#field_name();
                            self.#field_name = value;
                        }
                    }
                }
            }
        })
        .collect();

    // Generate _mut methods
    let mut_getters: Vec<_> = dirty_fields
        .iter()
        .map(|(field_name, field_type, _, _, field_vis)| {
            let mut_name = syn::Ident::new(&format!("{}_mut", field_name), field_name.span());

            quote! {
                #[inline]
                #field_vis fn #mut_name(&mut self) -> &mut #field_type {
                    self.__tracker |= Self::#field_name();
                    &mut self.#field_name
                }
            }
        })
        .collect();

    let expanded = quote! {
        #vis struct #name #generics {
            #(#new_fields),*
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #(#bit_consts)*
            #(#setters)*
            #(#mut_getters)*
        }
    };

    TokenStream::from(expanded)
}
