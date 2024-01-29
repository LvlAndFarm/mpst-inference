use std::{borrow::BorrowMut, fmt::Display, vec};
use lazy_static::lazy_static;
use parking_lot::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MPSTLocalType {
    Select(Participant, Vec<(String, MPSTLocalType)>),
    /* Branch is receive with external choice */
    Branch(Participant, Vec<(String, MPSTLocalType)>),
    RecX {
        cont: Box<MPSTLocalType>,
        id: i32,
        min_depth: Option<i32>,
        max_depth: Option<i32>,
    },
    // The bool is for whether the X has been "assigned" to a global recursive id. If it has, then we don't map it again.
    X(Option<i32>, bool),
    End
}

// Store a global static integer that increments whenever a new recursive type is created
lazy_static! {
    static ref RECURSIVE_COUNTER: Mutex<i32> = Mutex::new(0);
}

impl MPSTLocalType {
    pub fn receive(p: Participant, label: String, cont: MPSTLocalType) -> MPSTLocalType {
        Self::Branch(p, vec![(label, cont)])
    }

    pub fn send(p: Participant, label: String, cont: MPSTLocalType) -> MPSTLocalType {
        Self::Select(p, vec![(label, cont)])
    }

    pub fn recX(cont: Box<MPSTLocalType>) -> MPSTLocalType {
        *RECURSIVE_COUNTER.lock() += 1;
        Self::RecX {
            cont,
            id: *RECURSIVE_COUNTER.lock(),
            min_depth: None,
            max_depth: None,
        }
    }

    pub fn recX_with_id(cont: Box<MPSTLocalType>, id: i32) -> Self {
        Self::RecX {
            cont,
            id,
            min_depth: None,
            max_depth: None,
        }
    }

    pub fn x() -> Self {
        Self::X(None, false)
    }

    pub fn x_with_id(id: i32) -> Self {
        Self::X(Some(id), false)
    }

    pub fn map_local_x_to_global_rec(&self, local_id: i32, global_id: i32) -> Self {
        match self {
            MPSTLocalType::X(Some(id), false) if *id == local_id => {
                MPSTLocalType::X(Some(global_id), true)
            },
            MPSTLocalType::X(None, false) => {
                println!("WARNING: X should have been assigned a local id, assuming first recursive declaration");
                MPSTLocalType::X(Some(global_id), true)
            },
            MPSTLocalType::X(_, _) => self.clone(),
            MPSTLocalType::Select(p, choices) => {
                let mut new_choices: Vec<(String, MPSTLocalType)> = Vec::new();
                for (label, cont) in choices {
                    new_choices.push((label.clone(), cont.map_local_x_to_global_rec(local_id, global_id)));
                }
                MPSTLocalType::Select(p.clone(), new_choices)
            },
            MPSTLocalType::Branch(p, choices) => {
                let mut new_choices: Vec<(String, MPSTLocalType)> = Vec::new();
                for (label, cont) in choices {
                    new_choices.push((label.clone(), cont.map_local_x_to_global_rec(local_id, global_id)));
                }
                MPSTLocalType::Branch(p.clone(), new_choices)
            },
            MPSTLocalType::RecX {cont, id, min_depth, max_depth} => {
                MPSTLocalType::RecX { cont: Box::new(cont.map_local_x_to_global_rec(local_id, global_id)), id: *id, min_depth: *min_depth, max_depth: *max_depth }
            },
            MPSTLocalType::End => {
                MPSTLocalType::End
            }
        }
    }

    pub fn to_syn_ast(&self) -> syn::Expr {
        match self {
            MPSTLocalType::Select(participant, choices) => {
                println!("SEL Parse Start");
                let participant: syn::Expr = match &participant.role {
                    Some(role) => syn::parse_quote! { Some(String::from(#role)) },
                    None => syn::parse_quote! { None }
                };
                let mut syn_choices: Vec<syn::Expr> = Vec::new();
                for (label, ty) in choices {
                    let ty = ty.to_syn_ast();
                    syn_choices.push(
                        syn::parse_quote! {
                            (String::from(#label), #ty)
                        }
                    );
                }
                println!("SEL Parse End");
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::Select(
                        ::session::session_type::Participant::new(#participant),
                        vec![#(#syn_choices),*]
                    )
                }
            },
            MPSTLocalType::Branch(participant, choices) => {
                println!("BRANCH Parse Start");
                let participant: syn::Expr = match &participant.role {
                    Some(role) => syn::parse_quote! { Some(String::from(#role)) },
                    None => syn::parse_quote! { None }
                };
                let mut syn_choices: Vec<syn::Expr> = Vec::new();
                for (label, ty) in choices {
                    let ty = ty.to_syn_ast();
                    syn_choices.push(
                        syn::parse_quote! {
                            (String::from(#label), #ty)
                        }
                    );
                }
                println!("BRANCH Parse End");
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::Branch(
                        ::session::session_type::Participant::new(#participant),
                        vec![#(#syn_choices),*]
                    )
                }
            },
            MPSTLocalType::RecX {cont, id, min_depth, max_depth}=> {
                println!("REC Parse Start");
                let ty = cont.to_syn_ast();
                println!("REC Parse End");
                let min_depth: syn::Expr = option_to_ast(min_depth);
                let max_depth: syn::Expr = option_to_ast(max_depth);
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::RecX {cont: Box::new(#ty), id: #id, min_depth: #min_depth, max_depth: #max_depth}
                }
            },
            MPSTLocalType::X(depth, mapped) => {
                println!("X Parse Start");
                let depth: syn::Expr = match depth {
                    Some(depth) => syn::parse_quote! { Some(#depth) },
                    None => syn::parse_quote! { None }
                };
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::X(#depth, #mapped)
                }
            },
            MPSTLocalType::End => {
                println!("END Parse Start");
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::End
                }
            }
        }
    }
}

impl Display for MPSTLocalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MPSTLocalType::Select(participant, choices) => {
                write!(f, "Select<{}, {{", participant)?;
                for (label, cont) in choices {
                    write!(f, "{}.{}, ", label, cont)?;
                }
                write!(f, "}}")?;
            },
            MPSTLocalType::Branch(participant, choices) => {
                write!(f, "Branch<{}, {{", participant)?;
                for (label, cont) in choices {
                    write!(f, "{}.{}, ", label, cont)?;
                }
                write!(f, "}}")?;
            },
            MPSTLocalType::RecX{cont, id, min_depth, max_depth} => {
                write!(f, "Rec[{}]<{}>", id, cont)?;
            },
            MPSTLocalType::X(depth, _mapped) => write!(f, "X({:?}, {})", depth, _mapped)?,
            MPSTLocalType::End => write!(f, "End")?
        }
        Ok(())
    }
}

fn option_to_ast<T: quote::ToTokens>(opt: &Option<T>) -> syn::Expr {
    match opt {
        Some(val) => {
            let val_stream = val.to_token_stream();
            syn::parse_quote! { Some(#val_stream) }
        },
        None => syn::parse_quote! { None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Participant {
    role: Option<String>
}

impl Participant {
    pub fn new(role: Option<String>) -> Participant {
        Participant {
            role
        }
    }

    pub fn anonymous() -> Participant {
        Participant {
            role: None
        }
    }
}

impl Display for Participant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.role {
            Some(role) => write!(f, "{}", role),
            None => write!(f, "?")
        }
    }
}