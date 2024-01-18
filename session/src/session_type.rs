use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MPSTLocalType {
    Send(Participant, String, Box<MPSTLocalType>),
    Select(Participant, Vec<(String, MPSTLocalType)>),
    Receive(Participant, String, Box<MPSTLocalType>),
    /* Branch is receive with external choice */
    Branch(Participant, Vec<(String, MPSTLocalType)>),
    RecX(Box<MPSTLocalType>),
    X,
    End
}

impl Display for MPSTLocalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MPSTLocalType::Send(participant, label, cont) => {
                write!(f, "Send<{}, {}, {}>", participant, label, cont)?;
            },
            MPSTLocalType::Select(participant, choices) => {
                write!(f, "Select<{}, {{", participant)?;
                for (label, cont) in choices {
                    write!(f, "{}.{}, ", label, cont)?;
                }
                write!(f, "}}")?;
            },
            MPSTLocalType::Receive(participant, label, cont) => {
                write!(f, "Receive<{}, {}, {}>", participant, label, cont)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
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