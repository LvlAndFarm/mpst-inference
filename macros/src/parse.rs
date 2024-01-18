use syn::{punctuated::Punctuated, FnArg, token::Comma, spanned::Spanned};

use session::action::{LocalType, LocalType::*};

pub fn infer_block_session_type(item: &syn::Block) -> LocalType {
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

pub fn gen_session_type(expr: &syn::Expr, session_ident: &str) -> Result<Option<LocalType>, String> {
    println!("{:?}", expr.span().source_text());
    match expr {
        syn::Expr::Call(call) => {
            println!("Parsing call args for {:?}", call.func.span().source_text());
            let arg_psts: Result<Vec<_>, _> = dbg!(call.args.iter().map(|arg| gen_session_type(arg, session_ident)).collect());
            let arg_psts: Vec<LocalType> = arg_psts?.iter().filter_map(|arg| arg.clone()).collect();
            let arg_combined_pst = sequence_session_types(arg_psts);

            let call_pst = gen_session_type(&call.func, session_ident)?.unwrap_or(LocalType::End);

            Ok(Some(map_end_to(&call_pst, arg_combined_pst)))
        }
        syn::Expr::MethodCall(method_call) => {
            // Parse method call's argument local types
            println!("Parsing method call args for {:?}", method_call.method.to_string());
            let arg_psts: Result<Vec<_>, _> = dbg!(method_call.args.iter().map(|arg| gen_session_type(arg, session_ident)).collect());
            let arg_psts: Vec<LocalType> = arg_psts?.iter().filter_map(|arg| arg.clone()).collect();
            let arg_combined_pst = sequence_session_types(arg_psts);

            // Parse method call's receiver to send or receive
            let session_call: Result<Option<LocalType>, String> = if let syn::Expr::Path(path) = &*method_call.receiver {
                if let Some(ident) = path.path.get_ident() {
                    if ident == session_ident {
                        let method_name = method_call.method.to_string();
                            if method_name == "send" {
                                // We need to find label from the constructor of the first argument
                                let arg = method_call.args.first().ok_or("Invalid send call")?;
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
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            };

            Ok(Some(map_end_to(&arg_combined_pst, session_call?.unwrap_or(LocalType::End))))
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
        syn::Expr::ForLoop(for_expr) => {
            println!("Parsing for loop");
            let pat_type = gen_session_type(&for_expr.expr, session_ident)?.unwrap_or(End);
            let body_type = infer_block_session_type(&for_expr.body);
            let body_type_with_x = Box::new(map_end_to(&body_type, X));
            let block_type_with_choice = RecX(Box::new(InternalChoice(vec![body_type_with_x, Box::new(End)])));
            let block_with_pat = map_end_to(&pat_type, block_type_with_choice);
            Ok(Some(block_with_pat))
        },
        syn::Expr::Match(match_expr) => {
            println!("Parsing match");
            let mut session_choices = vec![];
            for arm in &match_expr.arms {
                match &arm.pat {
                    syn::Pat::TupleStruct(tuple_struct) => {
                        let label = tuple_struct.path.segments.last().unwrap().ident.to_string();
                        let cont = gen_session_type(&arm.body, session_ident)?.unwrap_or(End);
                        session_choices.push(Box::new(Receive(label, Box::new(cont))));
                    },
                    syn::Pat::Path(path) => {
                        let label = path.path.segments.last().unwrap().ident.to_string();
                        let cont = gen_session_type(&arm.body, session_ident)?.unwrap_or(End);
                        session_choices.push(Box::new(Receive(label, Box::new(cont))));
                    },
                    _ => {
                        return Err("Invalid match arm".to_string());
                    }
                }
                println!("{} => {}", arm.pat.span().unwrap().source_text().unwrap(), arm.body.span().unwrap().source_text().unwrap());
            }
            Ok(Some(ExternalChoice(session_choices)))
        },
        syn::Expr::Block(block) => {
            println!("Parsing block");
            Ok(Some(infer_block_session_type(&block.block)))
        },
        syn::Expr::Group(group) => {
            println!("Parsing group");
            gen_session_type(&group.expr, session_ident)
        },
        syn::Expr::Paren(paren) => {
            println!("Parsing paren");
            gen_session_type(&paren.expr, session_ident)
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