extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use syn::{punctuated::Punctuated, FnArg, token::Comma, spanned::Spanned};

mod action;
use action::{LocalType, LocalType::*};

#[proc_macro_attribute]
pub fn infer_session_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_ident = item.sig.ident.to_string();
    let output = infer_block_session_type(&item.block).to_string();
    let session_type_id = format_ident!("print_session_type_{}", fn_ident);
    println!("{}", session_type_id);
    (quote::quote! {
        fn #session_type_id () { 
            println!("{}", #output)
        }
    }).into()
    // (String::from("fn print_session_type() { println!(\"{}\", \"") + &output.to_string() + "\") }").parse().unwrap()
}


fn infer_block_session_type(item: &syn::Block) -> LocalType {
    let session_var = "s";
    let mut actions: Vec<LocalType> = vec![];
    for stmt in &item.stmts {
        match stmt {
            syn::Stmt::Expr(expr, _tok) => {
                if let Ok(Some(action)) = gen_session_type(expr, session_var) {
                    actions.push(action);
                }
            }
            _ => {}
        }
    }

    sequence_session_types(actions)
}

fn gen_session_type(expr: &syn::Expr, session_ident: &str) -> Result<Option<LocalType>, String> {
    println!("{:?}", expr.span().source_text().unwrap());
    match expr {
        syn::Expr::MethodCall(method_call) => {
            if let syn::Expr::Path(path) = &*method_call.receiver {
                if let Some(ident) = path.path.get_ident() {
                    if ident == session_ident {
                        let method_name = method_call.method.to_string();
                            if method_name == "send" {
                                // We need to find label from the constructor of the first argument
                                let arg = method_call.args.first().unwrap();
                                if let syn::Expr::Struct(struct_expr) = arg {
                                    let label = struct_expr.path.segments.first().unwrap().ident.to_string();
                                    return Ok(Some(Send(label, Box::new(End))));
                                } else if let syn::Expr::Path(path) = arg {
                                    let label = path.path.get_ident().unwrap().to_string();
                                    return Ok(Some(Send(label, Box::new(End))));
                                } else {
                                    return Err("Invalid send call".to_string());
                                }
                            } else if method_name == "receive" {
                                // We need to find label from the turbofish used in the method call
                                let turbofish = method_call.turbofish.as_ref().unwrap();
                                let label = turbofish.args.first().unwrap();
                                if let syn::GenericArgument::Type(ty) = label {
                                    if let syn::Type::Path(path) = ty {
                                        if let Some(ident) = path.path.get_ident() {
                                            return Ok(Some(Receive(ident.to_string(), Box::new(End))));
                                        } else {
                                            return Err("Invalid receive call".to_string());
                                        }
                                    } else {
                                        return Err("Invalid receive call".to_string());
                                    }
                                } else {
                                    return Err("Invalid receive call".to_string());
                                }
                            } else {
                                return Err("Invalid method call".to_string());
                            }
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        },
        syn::Expr::While(while_expr) => {
            println!("Parsing while loop");
            let cond_type = gen_session_type(&while_expr.cond, session_ident)?;
            println!("Cond type: {:?}", &cond_type);
            let body_type = infer_block_session_type(&while_expr.body);
            let body_type_with_x = Box::new(map_end_to(&body_type, X));
            let block_type_with_choice = InternalChoice(vec![body_type_with_x, Box::new(End)]);
            let block_with_cond =  if let Some(cond_type) = cond_type {
                map_end_to(&cond_type, block_type_with_choice)
            } else {
                block_type_with_choice
            };
            Ok(Some(RecX(Box::new(block_with_cond))))
        },
        _ => Ok(None)
    }
}

fn get_session_arg(args: &Punctuated<FnArg, Comma>) -> Option<String> {
    for arg in args {
        match arg {
            FnArg::Typed(pat_type) => {
                let pat = &*pat_type.pat;
                let ty = &*pat_type.ty;
                if let syn::Pat::Ident(ident) = pat {
                    if ident.ident == "session" {
                        if let syn::Type::Path(path) = ty {
                            if let Some(ident) = path.path.get_ident() {
                                return Some(ident.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn sequence_session_types(mut actions: Vec<LocalType>) -> LocalType {
    let mut session_type = End;
    actions.reverse();
    for action in actions {
        match action {
            Send(label, cont) => {
                session_type = Send(label, Box::new(map_end_to(&cont, session_type)))
            }
            Receive(label, cont) => {
                session_type = Receive(label, Box::new(map_end_to(&cont, session_type)))
            }
            RecX(cont) => {
                session_type = RecX(Box::new(map_end_to(&cont, session_type)))
            },
            InternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(Box::new(map_end_to(&choice, session_type.clone())));
                }
                session_type = InternalChoice(new_choices);
            },
            ExternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(Box::new(map_end_to(&choice, session_type.clone())));
                }
                session_type = ExternalChoice(new_choices);
            },
            End => panic!("Invalid session type"),
            X => panic!("Invalid session type")
        }
    }
    session_type
}

fn map_end_to(session_type: &LocalType, new_end: LocalType) -> LocalType {
    match session_type {
        End => new_end,
        Send(label, cont) => Send(label.clone(), Box::new(map_end_to(cont, new_end))),
        Receive(label, cont) => Receive(label.clone(), Box::new(map_end_to(cont, new_end))),
        RecX(cont) => RecX(Box::new(map_end_to(cont, new_end))),
        InternalChoice(choices) => {
            let mut new_choices = vec![];
            for choice in choices {
                new_choices.push(Box::new(map_end_to(&choice, new_end.clone())));
            }
            InternalChoice(new_choices)
        },
        ExternalChoice(choices) => {
            let mut new_choices = vec![];
            for choice in choices {
                new_choices.push(Box::new(map_end_to(&choice, new_end.clone())));
            }
            ExternalChoice(new_choices)
        },
        X => X
    }
}