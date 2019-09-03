extern crate proc_macro;

use quote::quote;
use std::collections::HashMap;
use syn::{Ident, Token, Type};

const STRUCT_MEMBER_SUFFIX: &str = "_IMPLEMENTATION_NAME_PLZ_DONT_USE_THIS_NAME_PLZ";
const ID_RESERVED_METHOD: &str = "id_ANOTHER_NAME_DONT_USE_EITHER_TY";
const DISPLAY_RESERVED_METHOD: &str = "display_ANOTHER_NAME_DONT_USE_EITHER_TY";

struct Item(Ident);
impl syn::parse::Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Item(input.parse::<Ident>()?))
    }
}

#[proc_macro]
pub fn create_provider_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let member = syn::parse_macro_input!(input as Item).0;

    let reserved_id_getter_name = Ident::new(
        &("id".to_owned() + ID_RESERVED_METHOD),
        proc_macro2::Span::call_site(),
    );
    let reserved_display_getter_name = Ident::new(
        &("display".to_owned() + DISPLAY_RESERVED_METHOD),
        proc_macro2::Span::call_site(),
    );
    let output = quote! {
        {
            let mut indexes = Vec::new();
            for translation_provider in #member {
               indexes.push((translation_provider.#reserved_id_getter_name(), translation_provider.#reserved_display_getter_name()));
            }
            indexes
        }
    };

    proc_macro::TokenStream::from(output)
}

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
    pub id: Ident,
    pub display: Ident,
}

impl syn::parse::Parse for TranslationParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut translation_keys = HashMap::new();
        let mut id = None;
        let mut display = None;
        while !input.is_empty() {
            let next_translation = input.parse::<syn::Ident>()?;

            let translation_params = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                content.parse::<TranslationParameters>()?
            } else {
                TranslationParameters::default()
            };

            translation_keys.insert(next_translation, translation_params);

            if input.peek(Token![=]) {
                input.parse::<Token![=]>().unwrap();
                let tag = input.parse::<syn::Ident>()?;
                match tag.to_string().as_str() {
                    "display" => display = Some(tag),
                    "id" => id = Some(tag),
                    _ => panic!("Unknown tag for the member - only 'display' and 'id' are supported for now!"),
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
        }

        Ok(TranslationParse {
            translation_keys,
            id: id.expect("Id member was not specified!"),
            display: display.expect("Display member was not specified!"),
        })
    }
}

#[proc_macro]
pub fn generate_translation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let translation_parse = syn::parse_macro_input!(input as TranslationParse);
    let id = translation_parse.id;
    let display = translation_parse.display;

    // generates the struct members
    let mut str_gen = quote! {};

    // generates the struc getters/formatters
    let reserved_id_getter_name = Ident::new(
        &("id".to_owned() + ID_RESERVED_METHOD),
        proc_macro2::Span::call_site(),
    );
    let reserved_display_getter_name = Ident::new(
        &("display".to_owned() + DISPLAY_RESERVED_METHOD),
        proc_macro2::Span::call_site(),
    );
    let mut fn_gen = quote! {
        pub fn #reserved_id_getter_name(&self) -> String {
            self.#id()
        }

        pub fn #reserved_display_getter_name(&self) -> String {
            self.#display()
        }
    };

    for (translation_key, translation_arguments) in translation_parse.translation_keys {
        // generates a unique field which is stored internally in the struct
        let internal_field_name = Ident::new(
            &(translation_key.to_string() + STRUCT_MEMBER_SUFFIX),
            proc_macro2::Span::call_site(),
        );
        let external_name = translation_key.to_string();

        // ... but we still want to ser/deser using the normal name!
        let serde_rename_field = if cfg!(feature = "serde") {
            quote! {
                #[serde(rename = #external_name)]
            }
        } else {
            quote! {}
        };

        str_gen = quote! {
            #str_gen
            #serde_rename_field
            #internal_field_name: String,
        };

        let mut param_decl = quote! { &self, };
        let mut param_call = quote! {};

        // We branch here because returning a Result<T, E> makes no sense if the translation is only a string
        // and not a formatting
        fn_gen = if translation_arguments.arguments.is_empty() {
            quote! {
                #fn_gen

                pub fn #translation_key ( #param_decl ) -> String {
                    self.#internal_field_name.clone()
                }
            }
        } else {
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
            quote! {
                #fn_gen

                pub fn #translation_key ( #param_decl ) -> Result<String, strfmt::FmtError> {
                    use std::collections::HashMap;
                    let mut params : HashMap<String, String> = HashMap::new();
                    #param_call
                    strfmt::strfmt(&self.#internal_field_name, &params)
                }
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
