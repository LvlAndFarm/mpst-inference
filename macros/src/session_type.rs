
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionType {
    Send(Participant, String, Box<SessionType>),
    Select(Participant, Vec<SessionType>),
    Receive(Participant, String, Box<SessionType>),
    /* Branch is receive with external choice */
    Branch(Participant, Vec<SessionType>),
    RecX(Box<SessionType>),
    X,
    End
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