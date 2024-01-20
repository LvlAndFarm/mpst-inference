extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::ToTokens;
use quote::quote;
use session::ilt::PartialLocalType;

mod parse;

#[proc_macro_attribute]
pub fn infer_session_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_ident = item.sig.ident.to_string();
    println!("Processing {}", fn_ident);
    let local_type = match parse::infer_block_session_type(&item.block).as_ref().map(PartialLocalType::to_local_type) {
        Ok(Ok(local_type)) => local_type,
        Ok(Err(err)) => panic!("Error: {}", err),
        Err(err) => panic!("Error: {}", err)
    };
    let ilt_tokens: proc_macro2::TokenStream = local_type.to_syn_ast().to_token_stream();
    println!("{}", ilt_tokens);
    let session_type_id = format_ident!("get_session_type_{}", fn_ident);

    let rumpsteak_session_type_id = format_ident!("get_rumpsteak_session_type_{}", fn_ident);
    let rumpsteak_session_type_tokens: proc_macro2::TokenStream = match local_type.to_session_type() {
        Ok(rs_type) => {
            println!("MPST output: {}", rs_type);
            let rs_type = rs_type.to_syn_ast();
            syn::parse_quote! {
                Ok(#rs_type)
            }
        },
        Err(err) => {
            syn::parse_quote! {
                Err(String::from(#err))
            }
        }
    };
    println!("{}", rumpsteak_session_type_tokens);

    println!("{}", session_type_id);
    (quote::quote! {
        #item

        fn #session_type_id () -> ::session::ilt::LocalType { 
            use ::session::ilt::LocalType;
            use ::session::ilt::LocalType::*;
            use ::session::session_type::Participant;

            #ilt_tokens
        }

        fn #rumpsteak_session_type_id () -> Result<::session::session_type::MPSTLocalType, String> {
            use ::session::session_type::Participant;
            use ::session::session_type::MPSTLocalType;
            use ::session::session_type::MPSTLocalType::*;

            #rumpsteak_session_type_tokens
        }
    }).into()
    // (String::from("fn print_session_type() { println!(\"{}\", \"") + &output.to_string() + "\") }").parse().unwrap()
}