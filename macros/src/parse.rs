use syn::{punctuated::Punctuated, FnArg, token::Comma, spanned::Spanned};

use session::ilt::{LocalType, PartialLocalType};

pub fn infer_block_session_type(item: &syn::Block) -> Result<PartialLocalType, String> {
    let session_var = "s";
    let mut actions: Vec<PartialLocalType> = vec![];
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

pub fn gen_session_type(expr: &syn::Expr, session_ident: &str) -> Result<Option<PartialLocalType>, String> {
    use PartialLocalType::*;
    // println!("{:?}", expr.span().source_text());
    match expr {
        syn::Expr::Call(call) => {
            // println!("Parsing call args for {:?}", call.func.span().source_text());
            let arg_psts: Result<Vec<Option<PartialLocalType>>, String> = call.args.iter().map(|arg| gen_session_type(arg, session_ident)).collect();
            let arg_psts: Vec<PartialLocalType> = arg_psts?.iter().filter_map(|arg| arg.clone()).collect();
            let arg_combined_pst = sequence_session_types(arg_psts)?;

            let call_pst = gen_session_type(&call.func, session_ident)?.unwrap_or(End);

            Ok(Some(call_pst.map_end_to(arg_combined_pst)))
        }
        syn::Expr::MethodCall(method_call) => {
            // Parse method call's argument local types
            // println!("Parsing method call args for {:?}", method_call.method.to_string());
            let arg_psts: Result<Vec<_>, _> = method_call.args.iter().map(|arg| gen_session_type(arg, session_ident)).collect();
            let arg_psts: Vec<PartialLocalType> = arg_psts?.iter().filter_map(|arg| arg.clone()).collect();
            let arg_combined_pst = sequence_session_types(arg_psts)?;

            // Parse method call's receiver to send or receive
            let session_call: Result<Option<PartialLocalType>, String> = if let syn::Expr::Path(path) = &*method_call.receiver {
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
                                    let label = path.path.segments.last().ok_or("Invalid Path in send call")?.ident.to_string();
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
                            } 
                            else if method_name == "branch" {
                                return Ok(None)
                            }
                            else {
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

            Ok(Some(arg_combined_pst.map_end_to(session_call?.unwrap_or(PartialLocalType::End))))
        },
        syn::Expr::While(while_expr) => {
            println!("Parsing while loop");
            let cond_type = gen_session_type(&while_expr.cond, session_ident)?;
            // println!("Cond type: {:?}", &cond_type);
            let body_type = infer_block_session_type(&while_expr.body)?;
            let body_type_with_x = body_type.map_end_to(X);
            let block_type_with_choice = InternalChoice(vec![body_type_with_x, End]);
            let block_with_cond =  if let Some(cond_type) = cond_type {
                cond_type.map_end_to(block_type_with_choice)
            } else {
                block_type_with_choice
            };
            Ok(Some(RecX(Box::new(block_with_cond))))
        },
        syn::Expr::ForLoop(for_expr) => {
            println!("Parsing for loop");
            let pat_type = gen_session_type(&for_expr.expr, session_ident)?.unwrap_or(End);
            let body_type = infer_block_session_type(&for_expr.body)?;
            let body_type_with_x = body_type.map_end_to(X);
            let block_type_with_choice = RecX(Box::new(InternalChoice(vec![body_type_with_x, End])));
            let block_with_pat = pat_type.map_end_to(block_type_with_choice);
            Ok(Some(block_with_pat))
        },
        syn::Expr::Match(match_expr) => {
            println!("Parsing match");

            let expr_type = gen_session_type(&match_expr.expr, session_ident)?.unwrap_or(End);

            println!("Parsed match expr type {:?}", expr_type);

            let mut session_choices = vec![];
            for arm in &match_expr.arms {
                match &arm.pat {
                    syn::Pat::TupleStruct(tuple_struct) => {
                        let label = tuple_struct.path.segments.last().unwrap().ident.to_string();
                        let cont = gen_session_type(&arm.body, session_ident)?.unwrap_or(End);
                        session_choices.push(Receive(label, Box::new(cont)));
                    },
                    syn::Pat::Path(path) => {
                        let label = path.path.segments.last().unwrap().ident.to_string();
                        let cont = gen_session_type(&arm.body, session_ident)?.unwrap_or(End);
                        session_choices.push(Receive(label, Box::new(cont)));
                    },
                    _ => {
                        return Err("Invalid match arm".to_string());
                    }
                }
                // println!("{} => {}", arm.pat.span().unwrap().source_text().unwrap(), arm.body.span().unwrap().source_text().unwrap());
            }
            Ok(Some(expr_type.map_end_to(ExternalChoice(session_choices))))
        },
        syn::Expr::If(if_expr) => {
            println!("Parsing if");
            let cond_type = gen_session_type(&if_expr.cond, session_ident)?;
            let then_type = infer_block_session_type(&if_expr.then_branch)?;
            let else_type = match &if_expr.else_branch {
                Some((_, else_block)) => gen_session_type(else_block.as_ref(), session_ident)?.unwrap_or(End),
                None => End
            };
            let if_type_with_choice = InternalChoice(vec![then_type, else_type]);
            let if_type_with_cond = if let Some(cond_type) = cond_type {
                cond_type.map_end_to(if_type_with_choice)
            } else {
                if_type_with_choice
            };
            Ok(Some(if_type_with_cond))
        },
        syn::Expr::Break(break_expr) => {
            println!("Parsing break");
            Ok(Some(Break))
        },
        syn::Expr::Loop(loop_expr) => {
            println!("Parsing loop");
            let body_type = infer_block_session_type(&loop_expr.body)?;
            let body_type_with_x = body_type.map_end_to(X);
            Ok(Some(RecX(Box::new(body_type_with_x))))
        },
        syn::Expr::Assign(assign_expr) => {
            println!("Parsing assign");
            let rhs_type = gen_session_type(&assign_expr.right, session_ident)?;
            let rhs_type = rhs_type.unwrap_or(End);
            Ok(Some(rhs_type))
        },
        syn::Expr::Block(block) => {
            println!("Parsing block");
            Ok(Some(infer_block_session_type(&block.block)?))
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

fn sequence_session_types(mut actions: Vec<PartialLocalType>) -> Result<PartialLocalType, String> {

    let mut session_type = PartialLocalType::End;
    actions.reverse();
    for action in actions {
        match action {
            PartialLocalType::Send(label, cont) => {
                session_type = PartialLocalType::Send(label, Box::new(cont.map_end_to(session_type.clone().into())))
            }
            PartialLocalType::Receive(label, cont) => {
                session_type = PartialLocalType::Receive(label, Box::new(cont.map_end_to(session_type.clone().into())))
            }
            PartialLocalType::RecX(cont) => {
                session_type = PartialLocalType::RecX(Box::new(cont.map_break_to(PartialLocalType::End).map_end_to(session_type.clone().into())))
            },
            PartialLocalType::InternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_end_to(session_type.clone().into()));
                }
                session_type = PartialLocalType::InternalChoice(new_choices);
            },
            PartialLocalType::ExternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_end_to(session_type.clone().into()));
                }
                session_type = PartialLocalType::ExternalChoice(new_choices);
            },
            PartialLocalType::End => (),
            PartialLocalType::X => {
                println!("Warning: X-recursion overriding rest of session type sequence");
                session_type = PartialLocalType::X;
            },
            PartialLocalType::Break => {
                println!("Warning: Break overriding rest of session type sequence");
                session_type = PartialLocalType::Break;
            }
        }
    }
    Ok(session_type)
}

// fn map_end_to(session_type: &LocalType, new_end: LocalType) -> LocalType {
//     match session_type {
//         End => new_end,
//         Send(label, cont) => Send(label.clone(), Box::new(map_end_to(cont, new_end))),
//         Receive(label, cont) => Receive(label.clone(), Box::new(map_end_to(cont, new_end))),
//         RecX(cont) => RecX(Box::new(map_end_to(cont, new_end))),
//         InternalChoice(choices) => {
//             let mut new_choices = vec![];
//             for choice in choices {
//                 new_choices.push(map_end_to(&choice, new_end.clone()));
//             }
//             InternalChoice(new_choices)
//         },
//         ExternalChoice(choices) => {
//             let mut new_choices = vec![];
//             for choice in choices {
//                 new_choices.push(map_end_to(&choice, new_end.clone()));
//             }
//             ExternalChoice(new_choices)
//         },
//         X => X
//     }
// }