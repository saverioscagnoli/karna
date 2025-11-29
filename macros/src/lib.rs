use darling::{FromDeriveInput, FromField, ast};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type, parse_macro_input};

// ============================================================================
// Getters Macro
// ============================================================================

#[derive(Debug, FromField)]
#[darling(attributes(get))]
struct GetterFieldOpts {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    mut_getter: bool,

    #[darling(default)]
    copied: bool,

    #[darling(default)]
    access: Option<String>,

    #[darling(default)]
    name: Option<String>,

    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct GetterOpts {
    ident: Ident,
    generics: syn::Generics, // Add this field
    data: ast::Data<(), GetterFieldOpts>,
}

#[proc_macro_derive(Getters, attributes(get))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match GetterOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = &opts.ident;
    let generics = &opts.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = opts.data.take_struct().unwrap();

    let getters = fields.iter().filter_map(|field| {
        if field.skip {
            return None;
        }

        let field_ident = field.ident.as_ref()?;
        let field_type = &field.ty;

        // Determine getter name
        let getter_name = if let Some(ref name) = field.name {
            Ident::new(name, field_ident.span())
        } else {
            field_ident.clone()
        };

        // Handle access path (for nested struct fields)
        let field_access = if let Some(ref access_path) = field.access {
            let parts: Vec<Ident> = access_path
                .split('.')
                .map(|s| Ident::new(s, field_ident.span()))
                .collect();
            quote! { self.#field_ident.#(#parts).* }
        } else {
            quote! { self.#field_ident }
        };

        // Generate immutable getter
        let immut_getter = if field.copied {
            quote! {
                pub fn #getter_name(&self) -> #field_type {
                    #field_access
                }
            }
        } else {
            quote! {
                pub fn #getter_name(&self) -> &#field_type {
                    &#field_access
                }
            }
        };

        // Generate mutable getter if requested
        let mut_getter = if field.mut_getter {
            let mut_getter_name = Ident::new(&format!("{}_mut", getter_name), getter_name.span());
            quote! {
                pub fn #mut_getter_name(&mut self) -> &mut #field_type {
                    &mut #field_access
                }
            }
        } else {
            quote! {}
        };

        Some(quote! {
            #immut_getter
            #mut_getter
        })
    });

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Setters Macro
// ============================================================================

#[derive(Debug, FromField)]
#[darling(attributes(set))]
struct SetterFieldOpts {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    access: Option<String>,

    #[darling(default)]
    into: bool,

    #[darling(default)]
    name: Option<String>,

    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct SetterOpts {
    ident: Ident,
    generics: syn::Generics, // Add this field
    data: ast::Data<(), SetterFieldOpts>,
}

#[proc_macro_derive(Setters, attributes(set))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match SetterOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = &opts.ident;
    let generics = &opts.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = opts.data.take_struct().unwrap();

    let setters = fields.iter().filter_map(|field| {
        if field.skip {
            return None;
        }

        let field_ident = field.ident.as_ref()?;
        let field_type = &field.ty;

        // Determine setter name
        let setter_name = if let Some(ref name) = field.name {
            Ident::new(name, field_ident.span())
        } else {
            Ident::new(&format!("set_{}", field_ident), field_ident.span())
        };

        // Handle access path (for nested struct fields)
        let field_access = if let Some(ref access_path) = field.access {
            let parts: Vec<Ident> = access_path
                .split('.')
                .map(|s| Ident::new(s, field_ident.span()))
                .collect();
            quote! { self.#field_ident.#(#parts).* }
        } else {
            quote! { self.#field_ident }
        };

        // Generate setter with or without Into conversion
        if field.into {
            Some(quote! {
                pub fn #setter_name(&mut self, value: impl Into<#field_type>) {
                    #field_access = value.into();
                }
            })
        } else {
            Some(quote! {
                pub fn #setter_name(&mut self, value: #field_type) {
                    #field_access = value;
                }
            })
        }
    });

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// With (Builder) Macro
// ============================================================================

#[derive(Debug, FromField)]
#[darling(attributes(with))]
struct WithFieldOpts {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    access: Option<String>,

    #[darling(default)]
    into: bool,

    #[darling(default)]
    name: Option<String>,

    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct WithOpts {
    ident: Ident,
    generics: syn::Generics, // Add this field
    data: ast::Data<(), WithFieldOpts>,
}

#[proc_macro_derive(With, attributes(with))]
pub fn derive_with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match WithOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = &opts.ident;
    let generics = &opts.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = opts.data.take_struct().unwrap();

    let with_methods = fields.iter().filter_map(|field| {
        if field.skip {
            return None;
        }

        let field_ident = field.ident.as_ref()?;
        let field_type = &field.ty;

        // Determine with method name
        let with_name = if let Some(ref name) = field.name {
            Ident::new(name, field_ident.span())
        } else {
            Ident::new(&format!("with_{}", field_ident), field_ident.span())
        };

        // Handle access path (for nested struct fields)
        let field_access = if let Some(ref access_path) = field.access {
            let parts: Vec<Ident> = access_path
                .split('.')
                .map(|s| Ident::new(s, field_ident.span()))
                .collect();
            quote! { self.#field_ident.#(#parts).* }
        } else {
            quote! { self.#field_ident }
        };

        // Generate builder method with or without Into conversion
        if field.into {
            Some(quote! {
                pub fn #with_name(mut self, value: impl Into<#field_type>) -> Self {
                    #field_access = value.into();
                    self
                }
            })
        } else {
            Some(quote! {
                pub fn #with_name(mut self, value: #field_type) -> Self {
                    #field_access = value;
                    self
                }
            })
        }
    });

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#with_methods)*
        }
    };

    TokenStream::from(expanded)
}
