use std::fmt::Display;

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