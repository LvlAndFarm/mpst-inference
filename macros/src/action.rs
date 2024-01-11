use std::{fmt::Display, string::ToString};

#[derive(Debug)]
pub enum LocalType {
    Send(String, Box<LocalType>),
    Receive(String, Box<LocalType>),
    RecX(Box<LocalType>),
    X,
    End
}

impl Display for LocalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalType::Send(label, ty) => write!(f, "Send(?, {}, {})", label, ty),
            LocalType::Receive(label, ty) => write!(f, "Receive(?, {}, {})", label, ty),
            LocalType::RecX(ty) => write!(f, "μX.{}", ty),
            LocalType::X => write!(f, "X"),
            LocalType::End => write!(f, "end")
        }
    }
}