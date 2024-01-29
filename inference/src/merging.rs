use std::{collections::{BTreeMap, BTreeSet, HashMap, HashSet}, fmt::Display, iter::Map};

use session::{session_type::{MPSTLocalType, Participant}, Message};

#[derive(Debug)]
pub enum GlobalType {
    Send(Participant, Participant, String, Box<GlobalType>),
    Select(Participant, Participant, Vec<(String, GlobalType)>),
    RecX(i32, Box<GlobalType>),
    X(i32),
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
            GlobalType::RecX(id, cont) => {
                write!(f, "Rec[{}]<{}>", id, cont)?;
            }
            GlobalType::X(id) => write!(f, "X[{}]", id)?,
            GlobalType::End => write!(f, "end")?,
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Parties {
    pub parties: BTreeMap<Participant, MPSTLocalType>,
    pub global_depth: i32,
    pub local_depth: BTreeMap<Participant, i32>,
    pub recursive_context: RecursiveContext,
}

#[derive(Clone)]
pub struct RecursiveContext {
    pub local_depths: BTreeMap<Participant, i32>,
    pub global_depth: i32
}

impl RecursiveContext {
    pub fn init(participants: &[Participant]) -> Self {
        RecursiveContext {
            local_depths: BTreeMap::from_iter(participants.iter().map(|p| (p.clone(), 0))),
            global_depth: 0,
        }
    }
}

impl Parties {
    pub fn new(parties: Vec<(Participant, MPSTLocalType)>) -> Self {
        let local_depth = BTreeMap::from_iter(parties.iter().map(|(p,_)| (p.clone(), 0)));
        Parties {
            parties: parties.clone().into_iter().collect(),
            global_depth: 0,
            local_depth,
            recursive_context: RecursiveContext::init(parties.iter().map(|(p,_)| p.clone()).collect::<Vec<_>>().as_slice()),
        }
    }

    pub fn is_end_state(&self) -> bool {
        self.parties.iter().all(|(_, lt)| matches!(lt, MPSTLocalType::End))
    }
}

impl Display for Parties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parties {{ ")?;
        for (p, lt) in &self.parties {
            write!(f, "{}: {}, ", p, lt)?;
        }
        write!(f, "}}")
    }
}

pub fn merge_locals(parties: Parties) -> Result<GlobalType, String> {
    println!("Merging local types {}", parties);
    if parties.is_end_state() {
        return Ok(GlobalType::End);
    }

    let (gen_new_rec, parties) = unwrap_rec(parties);
    if gen_new_rec {
        return Ok(GlobalType::RecX(parties.global_depth, Box::new(merge_locals(parties)?)));
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

    // The other reduction case is if all parties are End or compatible X(_), in which case we can just return X

    let (ends, non_ends): (Vec<_>, Vec<_>) = parties.parties.iter().partition(|(_, lt)| matches!(lt, MPSTLocalType::End));

    let mut all_end_or_x = true;
    let mut will_recurse_to = None;
    for (_, lt) in non_ends {
        match lt {
            MPSTLocalType::X(depth, mapped) => {
                assert!(mapped);
                match will_recurse_to {
                    Some(fixed_depth) => assert!(Some(fixed_depth) == *depth),
                    None => will_recurse_to = *depth,
                }
            },
            _ => {
                all_end_or_x = true;
                break
            },
        }
    }

    for (p, lt) in ends {
        match lt {
            MPSTLocalType::End => {
                match will_recurse_to {
                    None => (),
                    Some(depth) => if depth != parties.local_depth[p] {
                        all_end_or_x = false;
                        break
                    }
                }
            },
            _ => {
                unreachable!("Ends should be End")
            },
        }
    }
    if all_end_or_x {
        match will_recurse_to {
            None => return Ok(GlobalType::End),
            Some(depth) => return Ok(GlobalType::X(depth)),
        }
    }

    Err(format!("Cannot merge local types {}", parties))
}

fn unwrap_rec(parties: Parties) -> (bool, Parties) {
    // Iterate over all parties, and replace any RecX closures with their continuation.
    // If for any party, the local type is not RecX or End during the iteration, then we simply return a Result error
    // At the end of the iteration, we collect the results into a new Vec, and if we match an error, we return the original parties input.

    let mut gen_new_rec = false;
    let mut new_parties = Vec::with_capacity(parties.parties.len());
    for (p, lt) in &parties.parties {
        match lt {
            MPSTLocalType::RecX {cont, id, ..} => {
                new_parties.push((p.clone(), cont.map_local_x_to_global_rec(*id, parties.global_depth)));
                gen_new_rec = true;
                
            }
            MPSTLocalType::End => {
                new_parties.push((p.clone(), MPSTLocalType::End));
            }
            _ => return (false, parties),
        }
    }
    let mut new_parties = Parties {
        parties: new_parties.into_iter().collect(),
        global_depth: parties.global_depth,
        local_depth: parties.local_depth.clone(),
        recursive_context: parties.recursive_context,
    };
    if gen_new_rec {
        new_parties.recursive_context.global_depth = parties.global_depth;
        for (p, lt) in &new_parties.parties {
            match lt {
                MPSTLocalType::RecX {..} => {
                    new_parties.recursive_context.local_depths.insert(p.clone(), parties.local_depth[p]);
                }
                _ => (),
            }
        }
    }
    (gen_new_rec, new_parties)
}

fn reduce_then_merge(p1: Participant, p2: Participant, parties: &Parties) -> Result<GlobalType, String> {
    println!("Reducing {} and {} from {}", p1, p2, parties);
    let p1_mpst = parties.parties.iter().find(|(p, _)| *p == &p1).ok_or(String::from("Cannot find party 1 to reduce"))?.1.clone();
    let p2_mpst = parties.parties.iter().find(|(p, _)| *p == &p2).ok_or(String::from("Cannot find party 2 to reduce"))?.1.clone();
    match (p1_mpst, p2_mpst) {
        (MPSTLocalType::Branch(_, branch_conts), MPSTLocalType::Select(_, sel_conts)) => {
            let mut new_conts = Vec::new();
            for (label, sel_cont) in sel_conts {
                let (_, matched_branch_cont) = branch_conts.iter().find(|(label2, _)| label == *label2).ok_or(format!("No matching label for {}", label))?;
                
                let new_parties = parties.parties.iter().map(|(p, lt)| {
                    if p == &p1 {
                        (p.clone(), matched_branch_cont.clone())
                    } else if p == &p2 {
                        (p.clone(), sel_cont.clone())
                    } else {
                        (p.clone(), lt.clone())
                    }
                }).collect();
                let mut new_parties = Parties {
                    parties: new_parties,
                    global_depth: parties.global_depth,
                    local_depth: parties.local_depth.clone(),
                    recursive_context: parties.recursive_context.clone(),
                };
                new_parties.local_depth.insert(p1.clone(), parties.local_depth[&p1] + 1);
                new_parties.local_depth.insert(p2.clone(), parties.local_depth[&p2] + 1);
                new_parties.global_depth += 1;

                match merge_locals(new_parties) {
                    Ok(gt) => new_conts.push((label, gt)),
                    Err(_) => return Err(format!("Cannot merge local types {}", parties)),
                }
            }
            Ok(GlobalType::Select(p2, p1, new_conts))
        }
        (MPSTLocalType::Select(_, sel_conts), MPSTLocalType::Branch(_, branch_conts)) => {
            let mut new_conts = Vec::new();
            for (label, sel_cont) in sel_conts {
                let (_, matched_branch_cont) = branch_conts.iter().find(|(label2, _)| label == *label2).ok_or(format!("No matching label for {}", label))?;
                
                let new_parties = parties.parties.iter().map(|(p, lt)| {
                    if p == &p1 {
                        (p.clone(), sel_cont.clone())
                    } else if p == &p2 {
                        (p.clone(), matched_branch_cont.clone())
                    } else {
                        (p.clone(), lt.clone())
                    }
                }).collect();
                let mut new_parties = Parties {
                    parties: new_parties,
                    global_depth: parties.global_depth,
                    local_depth: parties.local_depth.clone(),
                    recursive_context: parties.recursive_context.clone()
                };

                new_parties.local_depth.insert(p1.clone(), parties.local_depth[&p1] + 1);
                new_parties.local_depth.insert(p2.clone(), parties.local_depth[&p2] + 1);
                new_parties.global_depth += 1;

                match merge_locals(new_parties) {
                    Ok(gt) => new_conts.push((label, gt)),
                    Err(_) => return Err(format!("Cannot merge local types {}", parties)),
                }
            }
            Ok(GlobalType::Select(p1, p2, new_conts))
        }
        _ => Err(format!("Cannot merge local types {}", parties))
    }
}

fn enumerate_duals(parties: &Parties) -> Vec<(Participant, Participant)> {
    let mut duals = Vec::new();
    let mut receivers: HashMap<String, Participant> = HashMap::new();
    let mut senders: HashMap<String, Participant> = HashMap::new();
    for (p1, local_type) in &parties.parties {
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
            MPSTLocalType::X(_, _) => (),
            MPSTLocalType::RecX {..} => (),
        }
    }
    duals
}