use std::{collections::HashMap, fmt::Display};

use session::{session_type::{MPSTLocalType, Participant}, Message};

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

pub fn merge_binary(
    lt1: (Participant, MPSTLocalType),
    lt2: (Participant, MPSTLocalType),
) -> Result<GlobalType, String> {
    println!("Merging ({}, {}) and ({}, {})", lt1.0, lt1.1, lt2.0, lt2.1);
    let (lt1_role, lt1) = lt1;
    let (lt2_role, lt2) = lt2;

    match (lt1, lt2) {
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
                    merge_binary(
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
                    merge_binary(
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
            Ok(GlobalType::RecX(Box::new(merge_binary(
                (lt1_role.clone(), *cont1),
                (lt2_role.clone(), *cont2),
            )?)))
        },
        (MPSTLocalType::X, MPSTLocalType::X) => Ok(GlobalType::X),
        (MPSTLocalType::End, MPSTLocalType::End) => Ok(GlobalType::End),
        _ => panic!("Incompatible types"),
    }
}

pub fn merge_locals(parties: Vec<(Participant, MPSTLocalType)>) -> Result<GlobalType, String> {
    println!("Merging local types {:?}", parties);
    if is_end_state(&parties) {
        return Ok(GlobalType::End);
    }

    let mut duals = enumerate_duals(&parties);
    duals.sort();
    duals.dedup();
    for (p1, p2) in duals {
        match reduce_then_merge(p1.clone(), p2.clone(), &parties) {
            Ok(gt) => return Ok(gt),
            Err(err) => println!("Cannot merge {} and {}: {}, trying next dual", p1, p2, err),
        }
    }
    Err(format!("Cannot merge local types {:?}", parties))
}

fn reduce_then_merge(p1: Participant, p2: Participant, parties: &Vec<(Participant, MPSTLocalType)>) -> Result<GlobalType, String> {
    println!("Reducing {} and {} from {:?}", p1, p2, parties);
    let p1_mpst = parties.iter().find(|(p, _)| p == &p1).ok_or(String::from("Cannot find party 1 to reduce"))?.1.clone();
    let p2_mpst = parties.iter().find(|(p, _)| p == &p2).ok_or(String::from("Cannot find party 2 to reduce"))?.1.clone();
    match (p1_mpst, p2_mpst) {
        (MPSTLocalType::Branch(_, branch_conts), MPSTLocalType::Select(_, sel_conts)) => {
            let mut new_conts = Vec::new();
            for (label, sel_cont) in sel_conts {
                let (_, matched_branch_cont) = branch_conts.iter().find(|(label2, _)| label == *label2).ok_or(format!("No matching label for {}", label))?;
                
                let new_parties = parties.iter().map(|(p, lt)| {
                    if p == &p1 {
                        (p.clone(), matched_branch_cont.clone())
                    } else if p == &p2 {
                        (p.clone(), sel_cont.clone())
                    } else {
                        (p.clone(), lt.clone())
                    }
                }).collect();

                match merge_locals(new_parties) {
                    Ok(gt) => new_conts.push((label, gt)),
                    Err(_) => return Err(format!("Cannot merge local types {:?}", parties)),
                }
            }
            Ok(GlobalType::Select(p2, p1, new_conts))
        }
        (MPSTLocalType::Select(_, sel_conts), MPSTLocalType::Branch(_, branch_conts)) => {
            let mut new_conts = Vec::new();
            for (label, sel_cont) in sel_conts {
                let (_, matched_branch_cont) = branch_conts.iter().find(|(label2, _)| label == *label2).ok_or(format!("No matching label for {}", label))?;
                
                let new_parties = parties.iter().map(|(p, lt)| {
                    if p == &p1 {
                        (p.clone(), sel_cont.clone())
                    } else if p == &p2 {
                        (p.clone(), matched_branch_cont.clone())
                    } else {
                        (p.clone(), lt.clone())
                    }
                }).collect();

                match merge_locals(new_parties) {
                    Ok(gt) => new_conts.push((label, gt)),
                    Err(_) => return Err(format!("Cannot merge SELECT {:?}", parties)),
                }
            }
            Ok(GlobalType::Select(p1, p2, new_conts))
        }
        _ => Err(format!("Cannot merge local types {:?}", parties))
    }
}

fn enumerate_duals(parties: &Vec<(Participant, MPSTLocalType)>) -> Vec<(Participant, Participant)> {
    let mut duals = Vec::new();
    let mut receivers: HashMap<String, Participant> = HashMap::new();
    let mut senders: HashMap<String, Participant> = HashMap::new();
    for (p1, local_type) in parties {
        match local_type {
            MPSTLocalType::Branch(p2, conts) => {
                for (label, _) in conts {
                    receivers.insert(label.clone(), p1.clone());

                    if senders.contains_key(label) {
                        duals.push((p1.clone(), senders[label].clone()));
                    }
                }
            }
            MPSTLocalType::Select(p2, conts) => {
                for (label, _) in conts {
                    senders.insert(label.clone(), p1.clone());

                    if receivers.contains_key(label) {
                        duals.push((p1.clone(), receivers[label].clone()));
                    }
                }
            }
            MPSTLocalType::End => (),
            _ => unimplemented!("Not implemented")
        }
    }
    duals
}

fn is_end_state(parties: &[(Participant, MPSTLocalType)]) -> bool {
    parties.iter().all(|(_, lt)| matches!(lt, MPSTLocalType::End))
}