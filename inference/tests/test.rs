use session::session_type::{MPSTLocalType, Participant};
use inference::merging::*;

#[test]
fn simple_merge() {
    let anon = Participant::anonymous();

    let lt1 = MPSTLocalType::Send(
        anon.clone(),
        String::from("Hello"),
        Box::new(MPSTLocalType::Branch(anon.clone(), vec![
            (String::from("Left"), MPSTLocalType::Receive(anon.clone(), String::from("LeftEnd"), Box::new(MPSTLocalType::End))),
            (String::from("Right"), MPSTLocalType::Send(anon.clone(), String::from("RightEnd"), Box::new(MPSTLocalType::End))),
        ]))
    );

    let lt1_role = Participant::new(Some(String::from("A")));

    let lt2 = MPSTLocalType::Receive(
        anon.clone(),
        String::from("Hello"),
        Box::new(MPSTLocalType::Select(anon.clone(), vec![
            (String::from("Left"), MPSTLocalType::Send(anon.clone(), String::from("LeftEnd"), Box::new(MPSTLocalType::End))),
            (String::from("Right"), MPSTLocalType::Receive(anon.clone(), String::from("RightEnd"), Box::new(MPSTLocalType::End))),
        ]))
    );
    
    let lt2_role = Participant::new(Some(String::from("B")));


    println!("{:?}", merge_locals((lt1_role, lt1), (lt2_role, lt2)).unwrap());
}