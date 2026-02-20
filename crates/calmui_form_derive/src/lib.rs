use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FormModel)]
pub fn derive_form_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            input.ident,
            "FormModel derive currently supports only non-generic structs",
        )
        .to_compile_error()
        .into();
    }

    let model_ident = input.ident;
    let fields_struct_ident = format_ident!("{model_ident}Fields");

    let named_fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return syn::Error::new(
                    Span::call_site(),
                    "FormModel derive requires a struct with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(
                Span::call_site(),
                "FormModel derive is only supported on structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let calmui = calmui_path();
    let mut lens_defs = Vec::new();
    let mut fields_methods = Vec::new();

    for field in named_fields {
        let Some(field_ident) = field.ident else {
            continue;
        };
        let field_ty = field.ty;
        let field_name = field_ident.to_string();
        let lens_ident = format_ident!("{model_ident}{}Lens", to_pascal_case(&field_name));

        lens_defs.push(quote! {
            #[derive(Clone, Copy, Debug, Default)]
            pub struct #lens_ident;

            impl #calmui::form::FieldLens<#model_ident> for #lens_ident {
                type Value = #field_ty;

                fn key(self) -> #calmui::form::FieldKey {
                    #calmui::form::FieldKey::new(#field_name)
                }

                fn get<'a>(self, model: &'a #model_ident) -> &'a Self::Value {
                    &model.#field_ident
                }

                fn set(self, model: &mut #model_ident, value: Self::Value) {
                    model.#field_ident = value;
                }
            }
        });

        fields_methods.push(quote! {
            pub const fn #field_ident(&self) -> #lens_ident {
                #lens_ident
            }
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct #fields_struct_ident;

        impl #fields_struct_ident {
            #(#fields_methods)*
        }

        impl #calmui::form::FormModel for #model_ident {
            type Fields = #fields_struct_ident;

            fn fields() -> Self::Fields {
                #fields_struct_ident
            }
        }

        #(#lens_defs)*
    }
    .into()
}

fn calmui_path() -> TokenStream2 {
    match crate_name("calmui") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Ok(FoundCrate::Itself) => quote!(crate),
        Err(_) => quote!(::calmui),
    }
}

fn to_pascal_case(input: &str) -> String {
    let mut out = String::new();
    for segment in input.split('_') {
        if segment.is_empty() {
            continue;
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}
