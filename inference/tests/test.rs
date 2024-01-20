use session::{session_type::{MPSTLocalType, Participant}, Session, Message};
use inference::merging::*;

#[test]
fn simple_merge_manual_types() {
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


    println!("{}", merge_locals((lt1_role, lt1), (lt2_role, lt2)).unwrap());
}

#[test]
fn simple_merge_inferred() {
    struct Hello;
    enum Choice1 { Left, Right };
    struct LeftEnd;
    struct RightEnd;

    impl Message for Hello {
        fn receive() -> Self {
            Hello
        }
    }

    impl Message for Choice1 {
        fn receive() -> Self {
            Choice1::Left
        }
    }

    impl Message for LeftEnd {
        fn receive() -> Self {
            LeftEnd
        }
    }

    impl Message for RightEnd {
        fn receive() -> Self {
            RightEnd
        }
    }


    #[macros::infer_session_type]
    fn client(mut s: Session) {
        s.send(Hello);
        match s.branch::<Choice1>() {
            Choice1::Left => {
                s.receive::<LeftEnd>();
            }
            Choice1::Right => {
                s.send(RightEnd);
            }
        }
    }

    #[macros::infer_session_type]
    fn server(mut s: Session) {
        s.receive::<Hello>();
        if 5 > 10 {
            s.send(Choice1::Left);
            s.send(LeftEnd);
        } else {
            s.send(Choice1::Right);
            s.receive::<RightEnd>();
        }
    }

    let client_role = Participant::new(Some(String::from("C")));
    let server_role = Participant::new(Some(String::from("S")));
    println!("Client.LocalType: {}", get_session_type_client());
    println!("Server.LocalType: {}", get_session_type_server());

    let client_mpst_local = get_rumpsteak_session_type_client().unwrap();
    let server_mpst_local = get_rumpsteak_session_type_server().unwrap();


    println!("{}", merge_locals((client_role, client_mpst_local), (server_role, server_mpst_local)).unwrap());
}


#[test]
fn more_general_branch() {
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
        Box::new(
            MPSTLocalType::Send(anon.clone(), String::from("Left"), 
                Box::new(MPSTLocalType::Send(anon.clone(), String::from("LeftEnd"), Box::new(MPSTLocalType::End)))
            )
        )
    );
    
    let lt2_role = Participant::new(Some(String::from("B")));


    println!("{}", merge_locals((lt1_role, lt1), (lt2_role, lt2)).unwrap());
}

#[test]
fn recursive_sum_manual() {
    /*
        Program 1:
        for i in 1..10 {
            s.send(Add);
        }
        s.send(Req);
        s.receive(Sum);

        Program 2:
        let mut sum = 0;
        while true {
            match s.receive() {
                Add => {
                    sum += 1;
                }
                Req => {
                    s.send(Sum(sum));
                    break;
                }
            }
        }
     */

    let anon = Participant::anonymous();

    let lt1 = MPSTLocalType::RecX(
        Box::new(
            MPSTLocalType::Select(anon.clone(), vec![
                (String::from("Add"), MPSTLocalType::X),
                (String::from("Req"), MPSTLocalType::Receive(anon.clone(), String::from("Ans"), Box::new(MPSTLocalType::End)))
            ])
        )
    );

    let lt1_role = Participant::new(Some(String::from("C")));

    let lt2 = MPSTLocalType::RecX(
        Box::new(
            MPSTLocalType::Branch(anon.clone(), vec![
                (String::from("Add"), MPSTLocalType::X),
                (String::from("Req"), MPSTLocalType::Send(anon.clone(), String::from("Ans"), Box::new(MPSTLocalType::End)))
            ])
        )
    );

    let lt2_role = Participant::new(Some(String::from("S")));

    println!("{}", merge_locals((lt1_role, lt1), (lt2_role, lt2)).unwrap());
}

#[test]
fn recursive_sum() {
    enum Choice1 {
        Add,
        Req
    }
    struct Sum;

    impl Message for Choice1 {
        fn receive() -> Self {
            Choice1::Add
        }
    }
    impl Message for Sum {
        fn receive() -> Self {
            Sum
        }
    }

    #[macros::infer_session_type]
    fn client(mut s: Session) {
        for _i in 1..10 {
            s.send(Choice1::Add);
        }
        s.send(Choice1::Req);
        s.receive::<Sum>();
    }

    #[macros::infer_session_type]
    fn server(mut s: Session) {
        loop {
            match s.branch::<Choice1>() {
                Choice1::Add => {
                    // sum += 1;
                }
                Choice1::Req => {
                    break;
                }
            }
        }
        s.send(Sum);
    }

    let client_role = Participant::new(Some(String::from("C")));
    let server_role = Participant::new(Some(String::from("S")));
    println!("Client.LocalType: {}", get_session_type_client());
    println!("Server.LocalType: {}", get_session_type_server());

    let client_mpst_local = get_rumpsteak_session_type_client().unwrap();
    let server_mpst_local = get_rumpsteak_session_type_server().unwrap();


    println!("{}", merge_locals((client_role, client_mpst_local), (server_role, server_mpst_local)).unwrap());
}