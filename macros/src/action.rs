use std::fmt::Display;

use crate::session_type::{SessionType, Participant};

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
    pub fn to_session_type(&self) -> Result<SessionType, String> {
        match self {
            LocalType::Send(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(SessionType::Send(Participant::anonymous(), label.clone(), Box::new(ty)))
            },
            LocalType::Receive(label, ty) => {
                let ty = ty.to_session_type()?;
                Ok(SessionType::Receive(Participant::anonymous(), label.clone(), Box::new(ty)))
            },
            LocalType::InternalChoice(choices) => {
                let mut session_choices = Vec::new();
                for choice in choices {
                    match choice.as_ref() {
                        LocalType::Send(label, cont) => {
                            let cont = cont.to_session_type()?;
                            session_choices.push(SessionType::Send(Participant::anonymous(), label.clone(), Box::new(cont)));
                        },
                        _ => {
                            return Err(format!("Internal choice must be followed by a send, found {}", choice));
                        }
                    }
                    session_choices.push(choice.to_session_type()?);
                }
                Ok(SessionType::Select(Participant::anonymous(), session_choices))
            },
            LocalType::ExternalChoice(choices) => {
                unimplemented!("External choice not implemented");
                // let mut session_choices = Vec::new();
                // for choice in choices {
                //     session_choices.push(choice.to_session_type()?);
                // }
                // Ok(SessionType::Branch(Participant::anonymous(), session_choices))
            },
            LocalType::RecX(ty) => {
                let ty = ty.to_session_type()?;
                Ok(SessionType::RecX(Box::new(ty)))
            },
            LocalType::X => Ok(SessionType::X),
            LocalType::End => Ok(SessionType::End)
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