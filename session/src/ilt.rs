use std::{fmt::Display, collections::HashSet};

use crate::session_type::{MPSTLocalType, Participant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalType {
    Send(String, Box<LocalType>),
    Receive(String, Box<LocalType>),
    InternalChoice(Vec<LocalType>),
    ExternalChoice(Vec<LocalType>),
    RecX(Box<LocalType>),
    X,
    End
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialLocalType {
    Send(String, Box<PartialLocalType>),
    Receive(String, Box<PartialLocalType>),
    InternalChoice(Vec<PartialLocalType>),
    ExternalChoice(Vec<PartialLocalType>),
    RecX(Box<PartialLocalType>),
    X,
    Break,
    End
}

impl PartialLocalType {
    pub fn map_break_to(&self, new_break: Self) -> Self {
        use PartialLocalType::*;

        match self {
            Break => new_break,
            Send(label, cont) => Send(label.clone(), Box::new(cont.map_break_to(new_break))),
            Receive(label, cont) => Receive(label.clone(), Box::new(cont.map_break_to(new_break))),
            RecX(cont) => RecX(Box::new(cont.map_break_to(new_break))),
            InternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_break_to(new_break.clone()));
                }
                InternalChoice(new_choices)
            },
            ExternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_break_to(new_break.clone()));
                }
                ExternalChoice(new_choices)
            },
            X => X,
            End => End
        }
    }

    pub fn map_end_to(&self, new_end: Self) -> Self {
        use PartialLocalType::*;

        match self {
            Break => Break,
            Send(label, cont) => Send(label.clone(), Box::new(cont.map_end_to(new_end))),
            Receive(label, cont) => Receive(label.clone(), Box::new(cont.map_end_to(new_end))),
            RecX(cont) => RecX(Box::new(cont.map_end_to(new_end))),
            InternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_end_to(new_end.clone()));
                }
                InternalChoice(new_choices)
            },
            ExternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(choice.map_end_to(new_end.clone()));
                }
                ExternalChoice(new_choices)
            },
            X => X,
            End => new_end
        }
    }

    pub fn of_local_type(ty: LocalType) -> Self {
        use PartialLocalType::*;

        match ty {
            LocalType::Send(label, cont) => Send(label, Box::new(PartialLocalType::of_local_type(*cont))),
            LocalType::Receive(label, cont) => Receive(label, Box::new(PartialLocalType::of_local_type(*cont))),
            LocalType::RecX(cont) => RecX(Box::new(PartialLocalType::of_local_type(*cont))),
            LocalType::InternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(PartialLocalType::of_local_type(choice));
                }
                InternalChoice(new_choices)
            },
            LocalType::ExternalChoice(choices) => {
                let mut new_choices = vec![];
                for choice in choices {
                    new_choices.push(PartialLocalType::of_local_type(choice));
                }
                ExternalChoice(new_choices)
            },
            LocalType::X => X,
            LocalType::End => End
        }
    }

    pub fn to_local_type(&self) -> Result<LocalType, String> {
        match self {
            PartialLocalType::Send(label, ty) => {
                let ty = ty.to_local_type()?;
                Ok(LocalType::Send(label.to_owned(), Box::new(ty)))
            },
            PartialLocalType::Receive(label, ty) => {
                let ty = ty.to_local_type()?;
                Ok(LocalType::Receive(label.to_owned(), Box::new(ty)))
            },
            PartialLocalType::InternalChoice(choices) => {
                let mut local_choices = Vec::new();
                for choice in choices {
                    local_choices.push(choice.to_local_type()?);
                }
                Ok(LocalType::InternalChoice(local_choices))
            },
            PartialLocalType::ExternalChoice(choices) => {
                let mut local_choices = Vec::new();
                for choice in choices {
                    local_choices.push(choice.to_local_type()?);
                }
                Ok(LocalType::ExternalChoice(local_choices))
            },
            PartialLocalType::RecX(ty) => {
                let ty = ty.to_local_type()?;
                Ok(LocalType::RecX(Box::new(ty)))
            },
            PartialLocalType::X => Ok(LocalType::X),
            PartialLocalType::Break => Err(String::from("Break is not a valid local type. Please remove before converting to local type")),
            PartialLocalType::End => Ok(LocalType::End)
        }
    }
}

impl From<LocalType> for PartialLocalType {
    fn from(ty: LocalType) -> Self {
        PartialLocalType::of_local_type(ty)
    }
}

impl LocalType {
    pub fn to_session_type(&self) -> Result<MPSTLocalType, String> {
        match self {
            LocalType::Send(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(MPSTLocalType::Select(Participant::anonymous(), vec![(label.clone(), ty)]))
            },
            LocalType::Receive(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(MPSTLocalType::Branch(Participant::anonymous(), vec![(label.clone(), ty)]))
            },
            LocalType::InternalChoice(choices) => {
                let mut session_choices = Vec::new();
                let mut unique_labels = HashSet::new();
                for choice in choices {
                    match choice {
                        LocalType::Send(label, cont) => {
                            if unique_labels.contains(label) {
                                return Err(format!("Label {} is not unique", label));
                            }
                            unique_labels.insert(label);
                            let cont = cont.to_session_type()?;
                            session_choices.push((label.to_owned(), cont));
                        },
                        _ => {
                            return Err(format!("Internal choice must be followed by a send, found {}", choice));
                        }
                    }
                }
                Ok(MPSTLocalType::Select(Participant::anonymous(), session_choices))
            },
            LocalType::ExternalChoice(choices) => {
                // unimplemented!("External choice not implemented");
                let mut session_choices = Vec::new();
                let mut unique_labels = HashSet::new();
                for choice in choices {
                    match choice {
                        LocalType::Receive(label, cont) => {
                            if unique_labels.contains(label) {
                                return Err(format!("Label {} is not unique", label));
                            }
                            unique_labels.insert(label);
                            let cont = cont.to_session_type()?;
                            session_choices.push((label.to_owned(), cont));
                        },
                        _ => {
                            return Err(format!("External choice must be followed by a receive, found {}", choice));
                        }
                    }
                }
                Ok(MPSTLocalType::Branch(Participant::anonymous(), session_choices))
            },
            LocalType::RecX(ty) => {
                let ty = ty.to_session_type()?;
                Ok(MPSTLocalType::RecX(Box::new(ty)))
            },
            LocalType::X => Ok(MPSTLocalType::X),
            LocalType::End => Ok(MPSTLocalType::End)
        }
    }

    pub fn to_syn_ast(&self) -> syn::Expr {
        match self {
            LocalType::Send(label, ty) => {
                let ty = ty.to_syn_ast();
                syn::parse_quote! {
                    ::session::ilt::LocalType::Send(String::from(#label), Box::new(#ty))
                }
            },
            LocalType::Receive(label, ty) => {
                let ty = ty.to_syn_ast();
                syn::parse_quote! {
                    ::session::ilt::LocalType::Receive(String::from(#label), Box::new(#ty))
                }
            },
            LocalType::InternalChoice(choices) => {
                let mut syn_choices = Vec::new();
                for choice in choices {
                    syn_choices.push(choice.to_syn_ast());
                }
                syn::parse_quote! {
                    ::session::ilt::LocalType::InternalChoice(vec![#(#syn_choices),*])
                }
            },
            LocalType::ExternalChoice(choices) => {
                let mut syn_choices = Vec::new();
                for choice in choices {
                    syn_choices.push(choice.to_syn_ast());
                }
                syn::parse_quote! {
                    ::session::ilt::LocalType::ExternalChoice(vec![#(#syn_choices),*])
                }
            },
            LocalType::RecX(ty) => {
                let ty = ty.to_syn_ast();
                syn::parse_quote! {
                    ::session::ilt::LocalType::RecX(Box::new(#ty))
                }
            },
            LocalType::X => syn::parse_quote! {
                ::session::ilt::LocalType::X
            },
            LocalType::End => syn::parse_quote! {
                ::session::ilt::LocalType::End
            }
        }
    }
}

impl Display for LocalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalType::Send(label, ty) => write!(f, "Send(?, {}, {})", label, ty),
            LocalType::Receive(label, ty) => write!(f, "Receive(?, {}, {})", label, ty),
            LocalType::RecX(ty) => write!(f, "μX.{}", ty),
            LocalType::X => write!(f, "X"),
            LocalType::End => write!(f, "end"),
            LocalType::InternalChoice(choices) => {
                let mut result = String::from("InternalChoice(");
                for choice in choices {
                    result.push_str(&format!("{}, ", choice));
                }
                result.push_str(")");
                write!(f, "{}", result)
            },
            LocalType::ExternalChoice(choices) => {
                let mut result = String::from("ExternalChoice(");
                for choice in choices {
                    result.push_str(&format!("{}, ", choice));
                }
                result.push_str(")");
                write!(f, "{}", result)
            }
        }
    }
}