use std::{fmt::Display, collections::HashSet};

use crate::session_type::{MPSTLocalType, Participant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalType {
    Send(String, Box<LocalType>),
    Receive(String, Box<LocalType>),
    InternalChoice(Vec<Box<LocalType>>),
    ExternalChoice(Vec<Box<LocalType>>),
    RecX(Box<LocalType>),
    X,
    End
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialLocalType {
    Send(String, Box<PartialLocalType>),
    Receive(String, Box<PartialLocalType>),
    InternalChoice(Vec<Box<PartialLocalType>>),
    ExternalChoice(Vec<Box<PartialLocalType>>),
    RecX(Box<PartialLocalType>),
    X,
    Break,
    End

}

impl LocalType {
    pub fn to_session_type(&self) -> Result<MPSTLocalType, String> {
        match self {
            LocalType::Send(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(MPSTLocalType::Send(Participant::anonymous(), label.clone(), Box::new(ty)))
            },
            LocalType::Receive(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(MPSTLocalType::Receive(Participant::anonymous(), label.clone(), Box::new(ty)))
            },
            LocalType::InternalChoice(choices) => {
                let mut session_choices = Vec::new();
                let mut unique_labels = HashSet::new();
                for choice in choices {
                    match choice.as_ref() {
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
                    match choice.as_ref() {
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