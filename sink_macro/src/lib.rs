use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(DataSink)]
pub fn sink_macro(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = input.ident;

    let expanded = quote! {
        impl #struct_name {
            pub fn boxed_new(config_path: &str) -> Box<dyn crate::sink::data_sink::DataSink> {
                Box::new(#struct_name::new(config_path))
            }
        }

        inventory::submit! {
            crate::sink::regist_info::RegistInfo {
                name: stringify!(#struct_name),
                constructor: #struct_name::boxed_new,
            }
        }
    };

    expanded.into()
}
