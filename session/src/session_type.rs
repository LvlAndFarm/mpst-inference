use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionType {
    Send(Participant, String, Box<SessionType>),
    Select(Participant, Vec<(String, SessionType)>),
    Receive(Participant, String, Box<SessionType>),
    /* Branch is receive with external choice */
    Branch(Participant, Vec<(String, SessionType)>),
    RecX(Box<SessionType>),
    X,
    End
}

impl Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Send(participant, label, cont) => {
                write!(f, "Send<{}, {}, {}>", participant, label, cont)?;
            },
            SessionType::Select(participant, choices) => {
                write!(f, "Select<{}, {{", participant)?;
                for (label, cont) in choices {
                    write!(f, "{}.{}, ", label, cont)?;
                }
                write!(f, "}}")?;
            },
            SessionType::Receive(participant, label, cont) => {
                write!(f, "Receive<{}, {}, {}>", participant, label, cont)?;
            },
            SessionType::Branch(participant, choices) => {
                write!(f, "Branch<{}, {{", participant)?;
                for (label, cont) in choices {
                    write!(f, "{}.{}, ", label, cont)?;
                }
                write!(f, "}}")?;
            },
            SessionType::RecX(cont) => {
                write!(f, "Rec<{}>", cont)?;
            },
            SessionType::X => write!(f, "X")?,
            SessionType::End => write!(f, "End")?
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