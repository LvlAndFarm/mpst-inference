use std::{fmt::Display, vec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MPSTLocalType {
    Select(Participant, Vec<(String, MPSTLocalType)>),
    /* Branch is receive with external choice */
    Branch(Participant, Vec<(String, MPSTLocalType)>),
    RecX(Box<MPSTLocalType>),
    X,
    End
}

impl MPSTLocalType {
    pub fn receive(p: Participant, label: String, cont: MPSTLocalType) -> MPSTLocalType {
        Self::Branch(p, vec![(label, cont)])
    }

    pub fn send(p: Participant, label: String, cont: MPSTLocalType) -> MPSTLocalType {
        Self::Select(p, vec![(label, cont)])
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
            MPSTLocalType::RecX(ty) => {
                println!("REC Parse Start");
                let ty = ty.to_syn_ast();
                println!("REC Parse End");
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::RecX(Box::new(#ty))
                }
            },
            MPSTLocalType::X => {
                println!("X Parse Start");
                syn::parse_quote! {
                    ::session::session_type::MPSTLocalType::X
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
            MPSTLocalType::RecX(cont) => {
                write!(f, "Rec<{}>", cont)?;
            },
            MPSTLocalType::X => write!(f, "X")?,
            MPSTLocalType::End => write!(f, "End")?
        }
        Ok(())
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