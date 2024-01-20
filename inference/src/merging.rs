use std::fmt::Display;

use session::session_type::{MPSTLocalType, Participant};

#[derive(Debug)]
pub enum GlobalType {
    Send(Participant, Participant, String, Box<GlobalType>),
    Select(Participant, Participant, Vec<(String, GlobalType)>),
    RecX(Box<GlobalType>),
    X,
    End,
}

impl Display for GlobalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalType::Send(from, to, label, cont) => {
                write!(f, "Send<{}, {}, {}, {}>", from, to, label, cont)?;
            }
            GlobalType::Select(from, to, choices) => {
                write!(f, "Select<{}, {}, {{ ", from, to)?;
                for (label, cont) in choices {
                    write!(f, "{}. {}, ", label, cont)?;
                }
                write!(f, "}}>")?;
            }
            GlobalType::RecX(cont) => {
                write!(f, "Rec<{}>", cont)?;
            }
            GlobalType::X => write!(f, "X")?,
            GlobalType::End => write!(f, "end")?,
        }
        Ok(())
    }
}

pub fn merge_locals(
    lt1: (Participant, MPSTLocalType),
    lt2: (Participant, MPSTLocalType),
) -> Result<GlobalType, String> {
    println!("Merging ({}, {}) and ({}, {})", lt1.0, lt1.1, lt2.0, lt2.1);
    let (lt1_role, lt1) = lt1;
    let (lt2_role, lt2) = lt2;

    // Replace Send with singular Select and Receive with singular Branch
    let lt1 = match lt1 {
        MPSTLocalType::Send(p, label, cont) => MPSTLocalType::Select(p, vec![(label, *cont)]),
        MPSTLocalType::Receive(p, label, cont) => MPSTLocalType::Branch(p, vec![(label, *cont)]),
        _ => lt1,
    };

    let lt2 = match lt2 {
        MPSTLocalType::Send(p, label, cont) => MPSTLocalType::Select(p, vec![(label, *cont)]),
        MPSTLocalType::Receive(p, label, cont) => MPSTLocalType::Branch(p, vec![(label, *cont)]),
        _ => lt2,
    };

    match (lt1, lt2) {
        (MPSTLocalType::Send(p1, label1, cont1), MPSTLocalType::Receive(p2, label2, cont2)) => {
            if p1 != p2 {
                return Err(format!("Incompatible participants: {} and {}", p1, p2));
            }

            if label1 != label2 {
                return Err(format!(
                    "Mismatched send and receive: {} and {}",
                    label1, label2
                ));
            }

            Ok(GlobalType::Send(
                lt1_role.clone(),
                lt2_role.clone(),
                label1,
                Box::new(merge_locals(
                    (lt1_role.clone(), *cont1),
                    (lt2_role.clone(), *cont2),
                )?),
            ))
        }

        (MPSTLocalType::Receive(p1, label1, cont1), MPSTLocalType::Send(p2, label2, cont2)) => {
            if p1 != p2 {
                return Err(format!("Incompatible participants: {} and {}", p1, p2));
            }

            if p1 != p2 {
                return Err(format!("Incompatible participants: {} and {}", p1, p2));
            }

            if label1 != label2 {
                return Err(format!(
                    "Mismatched send and receive: {} and {}",
                    label1, label2
                ));
            }

            Ok(GlobalType::Send(
                lt2_role.clone(),
                lt1_role.clone(),
                label1,
                Box::new(merge_locals(
                    (lt2_role.clone(), *cont1),
                    (lt1_role.clone(), *cont2),
                )?),
            ))
        }

        (MPSTLocalType::Branch(p1, rec_opts), MPSTLocalType::Select(p2, send_opts)) => {
            if p1 != p2 {
                return Err(format!("Incompatible participants: {} and {}", p1, p2));
            }

            let mut merged_opts = Vec::new();
            for (label1, cont1) in &send_opts {
                let (_, matched_rec_opt) = rec_opts
                    .iter()
                    .find(|(label2, _)| label1 == label2)
                    .ok_or(format!("No matching label for {}", label1))?;

                merged_opts.push((
                    label1.clone(),
                    merge_locals(
                        (lt2_role.clone(), cont1.clone()),
                        (lt1_role.clone(), matched_rec_opt.clone()),
                    )?,
                ));
            }

            Ok(GlobalType::Select(
                lt2_role.clone(),
                lt1_role.clone(),
                merged_opts,
            ))
        },

        (MPSTLocalType::Select(p2, send_opts), MPSTLocalType::Branch(p1, rec_opts)) => {
            if p1 != p2 {
                return Err(format!("Incompatible participants: {} and {}", p1, p2));
            }

            let mut merged_opts = Vec::new();
            for (label1, cont1) in &send_opts {
                let (_, matched_rec_opt) = rec_opts
                    .iter()
                    .find(|(label2, _)| label1 == label2)
                    .ok_or(format!("No matching label for {}", label1))?;

                merged_opts.push((
                    label1.clone(),
                    merge_locals(
                        (lt1_role.clone(), cont1.clone()),
                        (lt2_role.clone(), matched_rec_opt.clone()),
                    )?,
                ));
            }

            Ok(GlobalType::Select(
                lt1_role.clone(),
                lt2_role.clone(),
                merged_opts,
            ))
        },
        (MPSTLocalType::RecX(cont1), MPSTLocalType::RecX(cont2)) => {
            Ok(GlobalType::RecX(Box::new(merge_locals(
                (lt1_role.clone(), *cont1),
                (lt2_role.clone(), *cont2),
            )?)))
        },
        (MPSTLocalType::X, MPSTLocalType::X) => Ok(GlobalType::X),
        (MPSTLocalType::End, MPSTLocalType::End) => Ok(GlobalType::End),
        _ => panic!("Not implemented"),
    }
}
