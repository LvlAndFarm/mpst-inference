extern crate proc_macro;
use std::fmt::Error;

use proc_macro::TokenStream;
use syn::{parse::Parse, ItemFn, punctuated::Punctuated, FnArg, token::Comma, Local, spanned::Spanned};

mod action;
use action::{LocalType, LocalType::*};

#[proc_macro_attribute]
pub fn infer_session_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let output = infer_block_session_type(&item.block);
    (String::from("fn print_session_type() { println!(\"{}\", \"") + &output.to_string() + "\") }").parse().unwrap()
}


fn infer_block_session_type(item: &syn::Block) -> LocalType {
    let session_var = "s";
    let mut actions: Vec<LocalType> = vec![];
    for stmt in &item.stmts {
        match stmt {
            syn::Stmt::Expr(expr, tok) => {
                if let Ok(Some(action)) = gen_session_type(&expr, session_var) {
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
            let body_type_with_x = RecX(Box::new(map_end_to(&body_type, X)));
            println!("While body: {:?}", &body_type_with_x);
            if let Some(cond_type) = cond_type {
                return Ok(Some(map_end_to(&cond_type, body_type_with_x)));
            } else {
                return Ok(Some(body_type_with_x));
            }
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
                session_type = Send(label, Box::new(session_type))
            }
            Receive(label, cont) => {
                session_type = Receive(label, Box::new(session_type))
            }
            RecX(cont) => {
                session_type = RecX(Box::new(map_end_to(&cont, session_type)))
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
        X => X
    }
}