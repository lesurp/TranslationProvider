extern crate proc_macro;

use quote::quote;
use std::collections::HashMap;
use syn::{Ident, Token, Type};

struct TranslationParameters {
    pub arguments: Vec<(Ident, Type)>,
}

impl Default for TranslationParameters {
    fn default() -> Self {
        TranslationParameters {
            arguments: Vec::new(),
        }
    }
}

impl syn::parse::Parse for TranslationParameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut arguments = Vec::new();
        while !input.is_empty() {
            let param_name = input.parse::<syn::Ident>()?;
            input.parse::<Token![:]>()?;
            let type_name = input.parse::<syn::Type>()?;
            arguments.push((param_name, type_name));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
        }
        Ok(TranslationParameters { arguments })
    }
}

struct TranslationParse {
    pub translation_keys: HashMap<Ident, TranslationParameters>,
}

impl syn::parse::Parse for TranslationParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut translation_keys = HashMap::new();
        while !input.is_empty() {
            let next_translation = input.parse::<syn::Ident>()?;

            if input.peek(Token![,]) || input.is_empty() {
                translation_keys.insert(next_translation, TranslationParameters::default());
            } else {
                let content;
                syn::parenthesized!(content in input);
                translation_keys
                    .insert(next_translation, content.parse::<TranslationParameters>()?);
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
        }

        Ok(TranslationParse { translation_keys })
    }
}

#[proc_macro]
pub fn generate_translation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let translation_parse = syn::parse_macro_input!(input as TranslationParse);

    let mut str_gen = quote! {};
    let mut fn_gen = quote! {};

    for (translation_key, translation_arguments) in translation_parse.translation_keys {
        let method_name = Ident::new(
            &(translation_key.to_string() + "_"),
            proc_macro2::Span::call_site(),
        );
        str_gen = quote! {
            #str_gen
            #translation_key: String,
        };

        let mut param_decl = quote! { &self, };
        let mut param_call = quote! {};
        for (id, ty) in translation_arguments.arguments {
            param_decl = quote! {
                #param_decl
                #id: #ty,
            };

            param_call = quote! {
                #param_call
                params.insert(stringify!(#id).to_owned(), #id.to_string());
            };
        }

        fn_gen = quote! {
            #fn_gen

            pub fn #method_name ( #param_decl ) -> Result<String, strfmt::FmtError> {
                use std::collections::HashMap;
                let mut params : HashMap<String, String> = HashMap::new();
                #param_call
                strfmt::strfmt(&self.#translation_key, &params)
            }
        };
    }

    let serde_derive = if cfg!(feature = "serde") {
        quote! {
         #[derive(serde::Serialize, serde::Deserialize)]
        }
    } else {
        quote! {}
    };

    let final_generated_struct = quote! {
        #serde_derive
        pub struct TranslationProvider {
            #str_gen
        }

        impl TranslationProvider {
            #fn_gen
        }
    };

    let final_output = quote! {
        #final_generated_struct

        impl TranslationProvider {
            fn generated_code() -> String {
                stringify!(#final_generated_struct).to_owned()
            }
        }
    };

    proc_macro::TokenStream::from(final_output)
}
