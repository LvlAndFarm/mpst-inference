extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
mod action;

mod session_type;
mod parse;

#[proc_macro_attribute]
pub fn infer_session_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_ident = item.sig.ident.to_string();
    println!("Processing {}", fn_ident);
    let local_type = parse::infer_block_session_type(&item.block);
    let output = local_type.to_string();
    let session_type_id = format_ident!("session_type_{}", fn_ident);

    let rumpsteak_session_type_id = format_ident!("rumpsteak_session_type_{}", fn_ident);
    let rumpsteak_session_type = match local_type.to_session_type() {
        Ok(rs_type) => format!("{}", rs_type),
        Err(err) => format!("Error: {}", err)
    };
    println!("{}", session_type_id);
    (quote::quote! {
        fn #session_type_id () { 
            println!("{}", #output)
        }

        fn #rumpsteak_session_type_id () {
            println!("{}", #rumpsteak_session_type)
        }
    }).into()
    // (String::from("fn print_session_type() { println!(\"{}\", \"") + &output.to_string() + "\") }").parse().unwrap()
}