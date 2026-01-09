use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Fields, Ident, Lit, Meta, MetaNameValue, Token, Type, Visibility,
    parse::{Parse, ParseStream},
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

// Helper function to parse prop from string or ident
fn parse_prop(value: &Expr) -> Option<Vec<syn::Ident>> {
    match value {
        Expr::Lit(expr_lit) => {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                let parts: Vec<syn::Ident> = lit_str
                    .value()
                    .split('.')
                    .map(|s| syn::Ident::new(s.trim(), lit_str.span()))
                    .collect();
                Some(parts)
            } else {
                None
            }
        }
        Expr::Path(expr_path) => {
            if let Some(ident) = expr_path.path.get_ident() {
                Some(vec![ident.clone()])
            } else {
                None
            }
        }
        _ => None,
    }
}

// Helper function to parse type from string or expression
fn parse_type(value: &Expr) -> Option<Type> {
    match value {
        Expr::Lit(expr_lit) => {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                syn::parse_str(&lit_str.value()).ok()
            } else {
                None
            }
        }
        Expr::Path(expr_path) => Some(Type::Path(syn::TypePath {
            qself: None,
            path: expr_path.path.clone(),
        })),
        Expr::Reference(expr_ref) => {
            if let Expr::Path(ref_path) = &*expr_ref.expr {
                let mutability = if expr_ref.mutability.is_some() {
                    quote! { mut }
                } else {
                    quote! {}
                };
                let path = &ref_path.path;
                let ty_tokens = quote! { &#mutability #path };
                syn::parse2(ty_tokens).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

// Helper function to build property access chain
fn build_prop_access(field_name: &syn::Ident, props: &[syn::Ident]) -> proc_macro2::TokenStream {
    let mut access = quote! { self.#field_name };
    for prop in props {
        access = quote! { #access.#prop };
    }
    access
}

// Helper function to generate method name from field and props
fn generate_method_name(
    field_name: &syn::Ident,
    props: &Option<Vec<syn::Ident>>,
    prefix: &str,
    suffix: &str,
) -> syn::Ident {
    let base_name = if let Some(props) = props {
        format!(
            "{}_{}",
            field_name,
            props
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join("_")
        )
    } else {
        field_name.to_string()
    };

    let final_name = if !prefix.is_empty() && !suffix.is_empty() {
        format!("{}{}_{}", prefix, base_name, suffix)
    } else if !prefix.is_empty() {
        format!("{}{}", prefix, base_name)
    } else if !suffix.is_empty() {
        format!("{}_{}", base_name, suffix)
    } else {
        base_name
    };

    syn::Ident::new(&final_name, field_name.span())
}

struct GetConfig {
    prop: Option<Vec<syn::Ident>>,
    pre: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    name: Option<syn::Ident>,
    vis: Option<Visibility>,
    also: Option<Expr>,
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
        also: None,
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
                            config.prop = parse_prop(&value);
                        } else if path.is_ident("pre") {
                            if let Expr::Path(expr_path) = value {
                                config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                            }
                        } else if path.is_ident("cast") {
                            config.cast = parse_type(&value);
                        } else if path.is_ident("ty") {
                            config.ty = parse_type(&value);
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
                },
            }
        }
    }

    let method_name = config.name.clone().unwrap_or_else(|| {
        let suffix = if config.mutable { "mut" } else { "" };
        generate_method_name(field_name, &config.prop, "", suffix)
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

    let also_stmt = if let Some(also) = config.also {
        quote! { #also; }
    } else {
        quote! {}
    };

    let field_access = if let Some(props) = &config.prop {
        build_prop_access(field_name, props)
    } else {
        quote! { self.#field_name }
    };

    let body = if let Some(pre) = &config.pre {
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
    };

    quote! {
        #[inline]
        #vis fn #method_name(#self_arg) -> #return_type {
            #also_stmt
            #body
        }
    }
}

struct SetConfig {
    prop: Option<Vec<syn::Ident>>,
    pre: Option<syn::Ident>,
    cast: Option<Type>,
    ty: Option<Type>,
    from: Option<Type>,
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
        from: None,
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
                        config.prop = parse_prop(&value);
                    } else if path.is_ident("pre") {
                        if let Expr::Path(expr_path) = value {
                            config.pre = Some(expr_path.path.get_ident().unwrap().clone());
                        }
                    } else if path.is_ident("cast") {
                        config.cast = parse_type(&value);
                    } else if path.is_ident("ty") {
                        config.ty = parse_type(&value);
                    } else if path.is_ident("from") {
                        config.from = parse_type(&value);
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

    let method_name = config
        .name
        .clone()
        .unwrap_or_else(|| generate_method_name(field_name, &config.prop, "set_", ""));

    let vis = config.vis.unwrap_or_else(|| syn::parse_quote! { pub });

    // Determine param type and whether to use .into()
    let (param_type, use_into) = if let Some(from_ty) = &config.from {
        (quote! { #from_ty }, true)
    } else if let Some(ty) = &config.ty {
        if config.into {
            (quote! { impl Into<#ty> }, true)
        } else {
            (quote! { #ty }, false)
        }
    } else if config.into {
        (quote! { impl Into<#field_type> }, true)
    } else {
        (quote! { #field_type }, false)
    };

    let value_expr = if use_into {
        quote! { value.into() }
    } else {
        quote! { value }
    };

    let also_stmt = if let Some(also) = config.also {
        quote! { #also; }
    } else {
        quote! {}
    };

    let field_access = if let Some(props) = &config.prop {
        build_prop_access(field_name, props)
    } else {
        quote! { self.#field_name }
    };

    let assignment = if let Some(pre) = &config.pre {
        if let Some(cast_type) = &config.cast {
            quote! { #field_access.#pre() = #value_expr as #cast_type; }
        } else {
            quote! { #field_access.#pre() = #value_expr; }
        }
    } else if let Some(cast_type) = &config.cast {
        quote! { #field_access = #value_expr as #cast_type; }
    } else {
        quote! { #field_access = #value_expr; }
    };

    quote! {
        #[inline]
        #vis fn #method_name(&mut self, value: #param_type) {
            #also_stmt
            #assignment
        }
    }
}

struct WithConfig {
    prop: Option<Vec<syn::Ident>>,
    cast: Option<Type>,
    ty: Option<Type>,
    from: Option<Type>,
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
        from: None,
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
                        config.prop = parse_prop(&value);
                    } else if path.is_ident("cast") {
                        config.cast = parse_type(&value);
                    } else if path.is_ident("ty") {
                        config.ty = parse_type(&value);
                    } else if path.is_ident("from") {
                        config.from = parse_type(&value);
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

    let method_name = config
        .name
        .clone()
        .unwrap_or_else(|| generate_method_name(field_name, &config.prop, "with_", ""));

    let vis = config.vis.unwrap_or_else(|| syn::parse_quote! { pub });

    // Determine param type and whether to use .into()
    let (param_type, use_into) = if let Some(from_ty) = &config.from {
        (quote! { #from_ty }, true)
    } else if let Some(ty) = &config.ty {
        if config.into {
            (quote! { impl Into<#ty> }, true)
        } else {
            (quote! { #ty }, false)
        }
    } else if config.into {
        (quote! { impl Into<#field_type> }, true)
    } else {
        (quote! { #field_type }, false)
    };

    let value_expr = if use_into {
        quote! { value.into() }
    } else {
        quote! { value }
    };

    let also_stmt = if let Some(also) = config.also {
        quote! { #also; }
    } else {
        quote! {}
    };

    let field_access = if let Some(props) = &config.prop {
        build_prop_access(field_name, props)
    } else {
        quote! { self.#field_name }
    };

    // Generate a compile-time assertion if `from` is used
    let from_assertion = if let Some(from_ty) = &config.from {
        let target_ty = config.ty.as_ref().unwrap_or(field_type);
        quote! {
            const _: fn() = || {
                fn assert_impl<T: From<U>, U>() {}
                assert_impl::<#target_ty, #from_ty>();
            };
        }
    } else {
        quote! {}
    };

    let assignment = if let Some(cast_type) = &config.cast {
        quote! { #field_access = #value_expr as #cast_type; }
    } else {
        quote! { #field_access = #value_expr; }
    };

    quote! {
        #from_assertion
        #[inline]
        #vis fn #method_name(mut self, value: #param_type) -> Self {
            #also_stmt
            #assignment
            self
        }
    }
}

/// Derive the random function for enums with fieldless variants
///
/// Just returns a random variant of the enum. Ignores fieldful variants.
#[proc_macro_derive(Random)]
pub fn derive_random(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure it's an enum
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("Random can only be derived for enums"),
    };

    // Collect only fieldless variant names
    let variant_names: Vec<_> = variants
        .iter()
        .filter(|v| matches!(v.fields, Fields::Unit))
        .map(|v| &v.ident)
        .collect();
    let variant_count = variant_names.len();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn random() -> Self {
                let variants = [
                    #(Self::#variant_names,)*
                ];
                variants[rand::random_range(0..#variant_count)]
            }
        }
    };

    TokenStream::from(expanded)
}

struct TrackerType {
    ty: syn::Type,
}

impl Parse for TrackerType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TrackerType { ty: input.parse()? })
    }
}

#[proc_macro_attribute]
pub fn track_dirty(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Parse tracker type from attribute, default to u8
    let tracker_type = if attr.is_empty() {
        syn::parse_str::<syn::Type>("u8").unwrap()
    } else {
        match syn::parse::<TrackerType>(attr) {
            Ok(t) => t.ty,
            Err(_) => {
                return syn::Error::new_spanned(
                    &input,
                    "expected a type like u8, u16, u32, u64, or u128",
                )
                .to_compile_error()
                .into();
            }
        }
    };

    // Determine max fields based on type
    let max_fields = match tracker_type {
        syn::Type::Path(ref p) if p.path.is_ident("u8") => 8,
        syn::Type::Path(ref p) if p.path.is_ident("u16") => 16,
        syn::Type::Path(ref p) if p.path.is_ident("u32") => 32,
        syn::Type::Path(ref p) if p.path.is_ident("u64") => 64,
        syn::Type::Path(ref p) if p.path.is_ident("u128") => 128,
        _ => {
            return syn::Error::new_spanned(
                &tracker_type,
                "tracker type must be one of: u8, u16, u32, u64, u128",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(&input, "only named fields are supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    // Check field count
    if fields.len() > max_fields {
        return syn::Error::new_spanned(
            &input,
            format!(
                "too many fields ({}) for {} tracker (maximum {} fields)",
                fields.len(),
                quote!(#tracker_type),
                max_fields
            ),
        )
        .to_compile_error()
        .into();
    }

    // Generate _f() functions for each field
    let field_fns = fields.iter().enumerate().map(|(i, field)| {
        let field_name = field.ident.as_ref().unwrap();
        let fn_name = Ident::new(&format!("{}_f", field_name), proc_macro2::Span::call_site());
        let shift = i as u32;
        quote! {
            #[inline]
            pub const fn #fn_name() -> #tracker_type {
                1 << #shift
            }
        }
    });

    // Calculate the "all dirty" mask
    let all_mask = if fields.is_empty() {
        quote!(0)
    } else {
        let field_count = fields.len() as u32;
        quote!((1 << #field_count) - 1)
    };

    // Reconstruct existing fields
    let existing_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let ty = &f.ty;
        let field_vis = &f.vis;
        let field_attrs = &f.attrs;
        quote! {
            #(#field_attrs)*
            #field_vis #field_name: #ty
        }
    });

    let expanded = quote! {
        #(#attrs)*
        #vis struct #name #generics {
            #(#existing_fields,)*
            pub tracker: #tracker_type,
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #(#field_fns)*

            /// Check if a specific dirty flag is set
            #[inline]
            pub const fn is_dirty(&self, flag: #tracker_type) -> bool {
                self.tracker & flag != 0
            }

            /// Set a specific dirty flag
            #[inline]
            pub fn set_dirty(&mut self, flag: #tracker_type) {
                self.tracker |= flag;
            }

            /// Clear a specific dirty flag
            #[inline]
            pub fn clear_dirty(&mut self, flag: #tracker_type) {
                self.tracker &= !flag;
            }

            /// Set all fields as dirty
            #[inline]
            pub fn set_all_dirty(&mut self) {
                self.tracker = #all_mask;
            }

            /// Clear all dirty flags
            #[inline]
            pub fn clear_all_dirty(&mut self) {
                self.tracker = 0;
            }

            /// Check if any field is dirty
            #[inline]
            pub const fn any_dirty(&self) -> bool {
                self.tracker != 0
            }
        }
    };

    TokenStream::from(expanded)
}
