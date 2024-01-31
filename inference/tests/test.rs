use session::{session_type::{MPSTLocalType, Participant}, Session, Message};
use inference::merging::*;

#[test]
fn simple_merge_manual_types() {
    let anon = Participant::anonymous();

    let lt1 = MPSTLocalType::send(
        anon.clone(),
        String::from("Hello"),
        MPSTLocalType::Branch(anon.clone(), vec![
            (String::from("Left"), MPSTLocalType::receive(anon.clone(), String::from("LeftEnd"), MPSTLocalType::End)),
            (String::from("Right"), MPSTLocalType::send(anon.clone(), String::from("RightEnd"), MPSTLocalType::End)),
        ])
    );

    let lt1_role = Participant::new(Some(String::from("A")));

    let lt2 = MPSTLocalType::receive(
        anon.clone(),
        String::from("Hello"),
        MPSTLocalType::Select(anon.clone(), vec![
            (String::from("Left"), MPSTLocalType::send(anon.clone(), String::from("LeftEnd"), MPSTLocalType::End)),
            (String::from("Right"), MPSTLocalType::receive(anon.clone(), String::from("RightEnd"), MPSTLocalType::End)),
        ])
    );
    
    let lt2_role = Participant::new(Some(String::from("B")));


    println!("{}", merge_locals(Parties::new(vec![(lt1_role, lt1), (lt2_role, lt2)])).unwrap());
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


    println!("{}", merge_locals(Parties::new(vec![(client_role, client_mpst_local), (server_role, server_mpst_local)])).unwrap());
}


#[test]
fn more_general_branch() {
    let anon = Participant::anonymous();

    let lt1 = MPSTLocalType::send(
        anon.clone(),
        String::from("Hello"),
        MPSTLocalType::Branch(anon.clone(), vec![
            (String::from("Left"), MPSTLocalType::receive(anon.clone(), String::from("LeftEnd"), MPSTLocalType::End)),
            (String::from("Right"), MPSTLocalType::send(anon.clone(), String::from("RightEnd"), MPSTLocalType::End)),
        ])
    );

    let lt1_role = Participant::new(Some(String::from("A")));

    let lt2 = MPSTLocalType::receive(
        anon.clone(),
        String::from("Hello"),
        MPSTLocalType::send(anon.clone(), String::from("Left"), 
            MPSTLocalType::send(anon.clone(), String::from("LeftEnd"), MPSTLocalType::End)
        )
    );
    
    let lt2_role = Participant::new(Some(String::from("B")));


    println!("{}", merge_locals(Parties::new(vec![(lt1_role, lt1), (lt2_role, lt2)])).unwrap());
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

    let lt1 = MPSTLocalType::recX(
        Box::new(
            MPSTLocalType::Select(anon.clone(), vec![
                (String::from("Add"), MPSTLocalType::x()),
                (String::from("Req"), MPSTLocalType::Branch(anon.clone(), vec![(String::from("Ans"), MPSTLocalType::End)]))
            ])
        )
    );

    let lt1_role = Participant::new(Some(String::from("C")));

    let lt2 = MPSTLocalType::recX(
        Box::new(
            MPSTLocalType::Branch(anon.clone(), vec![
                (String::from("Add"), MPSTLocalType::x()),
                (String::from("Req"), MPSTLocalType::Select(anon.clone(), vec![(String::from("Ans"), MPSTLocalType::End)]))
            ])
        )
    );

    let lt2_role = Participant::new(Some(String::from("S")));

    println!("{}", merge_locals(Parties::new(vec![(lt1_role, lt1), (lt2_role, lt2)])).unwrap());
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


    println!("{}", merge_locals(Parties::new(vec![(client_role, client_mpst_local), (server_role, server_mpst_local)])).unwrap());
}

#[test]
fn test_triple_session_type() {
    struct Hello;
    enum Choice {
        Left,
        Right
    }
    enum Choice2 {
        CLeft,
        CRight
    }

    impl Message for Hello {
        fn receive() -> Self {
            Hello
        }
    }

    impl Message for Choice {
        fn receive() -> Self {
            Choice::Left
        }
    }

    impl Message for Choice2 {
        fn receive() -> Self {
            Choice2::CLeft
        }
    }

    #[macros::infer_session_type]
    fn A(mut s: Session) {
        s.send(Hello);
        if env!("PATH").contains("debug") {
            s.send(Choice::Left);
        } else {
            s.send(Choice::Right);
        }
    }

    #[macros::infer_session_type]
    fn B(mut s: Session) {
        s.receive::<Hello>();
        match s.branch::<Choice>() {
            Choice::Left => {
                s.send(Choice2::CLeft);
            }
            Choice::Right => {
                s.send(Choice2::CRight);
            }
        }
    }

    #[macros::infer_session_type]
    fn C(mut s: Session) {
        match s.branch::<Choice2>() {
            Choice2::CLeft => {
                println!("Left");
            }
            Choice2::CRight => {
                println!("Right");
            }
        }
    }

    let a_role = Participant::new(Some(String::from("A")));
    let b_role = Participant::new(Some(String::from("B")));
    let c_role = Participant::new(Some(String::from("C")));

    let a_mpst_local = get_rumpsteak_session_type_A().unwrap();
    let b_mpst_local = get_rumpsteak_session_type_B().unwrap();
    let c_mpst_local = get_rumpsteak_session_type_C().unwrap();

    println!("A.MPSTLocalType: {}", a_mpst_local);
    println!("B.MPSTLocalType: {}", b_mpst_local);
    println!("C.MPSTLocalType: {}", c_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(a_role, a_mpst_local), (b_role, b_mpst_local), (c_role, c_mpst_local)])).unwrap());
}

#[test]
fn test_backtracking_triple() {
    /**
     * Expected GlobalType: Select<A, B, { Hello. Select<A, B, { Left. Select<B, C, { Left. end, }>, Right. Select<B, C, { Right. end, }>, }>, }>
     * Expected LocalType A: Send<?, Hello, Select<?, {Left.End, Right.End, }>
     * Expected LocalType B: Receive<?, Hello, Branch<?, {Left.Send<?, Left, End>, Right.Send<?, Right, End>, }>
     * Expected LocalType C: Branch<?, {Left.End, Right.End, }
     * 
     * This is a test for backtracking. The initial duals are <A, B> and <A, C>, but <A, C> leads to a dead end, so we backtrack to <A, B>.
     * The backtracking behaviour might not be triggered if <A, B> is explored first.
     */
    enum Choice {
        Left,
        Right
    }

    impl Message for Choice {
        fn receive() -> Self {
            Choice::Left
        }
    }

    #[macros::infer_session_type]
    fn A(mut s: Session) {
        if env!("PATH").contains("debug") {
            s.send(Choice::Left);
        } else {
            s.send(Choice::Right);
        }
    }

    #[macros::infer_session_type]
    fn B(mut s: Session) {
        match s.branch::<Choice>() {
            Choice::Left => {
                s.send(Choice::Left);
            }
            Choice::Right => {
                s.send(Choice::Right);
            }
        }
    }

    #[macros::infer_session_type]
    fn C(mut s: Session) {
        match s.branch::<Choice>() {
            Choice::Left => {
                println!("Left");
            }
            Choice::Right => {
                println!("Right");
            }
        }
    }

    let a_role = Participant::new(Some(String::from("A")));
    let b_role = Participant::new(Some(String::from("B")));
    let c_role = Participant::new(Some(String::from("C")));

    let a_mpst_local = get_rumpsteak_session_type_A().unwrap();
    let b_mpst_local = get_rumpsteak_session_type_B().unwrap();
    let c_mpst_local = get_rumpsteak_session_type_C().unwrap();

    println!("A.MPSTLocalType: {}", a_mpst_local);
    println!("B.MPSTLocalType: {}", b_mpst_local);
    println!("C.MPSTLocalType: {}", c_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(a_role, a_mpst_local), (c_role, c_mpst_local), (b_role, b_mpst_local)])).unwrap());
}

#[test]
fn test_recursive_triple() {
    struct Msg1;
    struct Msg2;

    impl Message for Msg1 {
        fn receive() -> Self {
            Msg1
        }
    }

    impl Message for Msg2 {
        fn receive() -> Self {
            Msg2
        }
    }

    #[macros::infer_session_type]
    fn A(mut s: Session) {
        loop {
            s.send(Msg1);
            s.send(Msg2);
        }
    }

    #[macros::infer_session_type]
    fn B(mut s: Session) {
        loop {
            s.receive::<Msg1>();
        }
    }

    #[macros::infer_session_type]
    fn C(mut s: Session) {
        loop {
            s.receive::<Msg2>();
        }
    }

    let a_role = Participant::new(Some(String::from("A")));
    let b_role = Participant::new(Some(String::from("B")));
    let c_role = Participant::new(Some(String::from("C")));

    let a_mpst_local = get_rumpsteak_session_type_A().unwrap();
    let b_mpst_local = get_rumpsteak_session_type_B().unwrap();
    let c_mpst_local = get_rumpsteak_session_type_C().unwrap();

    println!("A.MPSTLocalType: {}", a_mpst_local);
    println!("B.MPSTLocalType: {}", b_mpst_local);
    println!("C.MPSTLocalType: {}", c_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(a_role, a_mpst_local), (b_role, b_mpst_local), (c_role, c_mpst_local)])).unwrap());
}

#[test]
fn eventually_synchronous_mpst() {
    struct Hello;
    struct Repeat1;
    struct Repeat2;

    impl Message for Hello {
        fn receive() -> Self {
            Hello
        }
    }

    impl Message for Repeat1 {
        fn receive() -> Self {
            Repeat1
        }
    }

    impl Message for Repeat2 {
        fn receive() -> Self {
            Repeat2
        }
    }

    #[macros::infer_session_type]
    fn A(mut s: Session) {
        s.send(Hello);
        loop {
            s.send(Repeat1);
        }
    }

    #[macros::infer_session_type]
    fn B(mut s: Session) {
        s.receive::<Hello>();
        loop {
            s.receive::<Repeat1>();
        }
    }

    #[macros::infer_session_type]
    fn C(mut s: Session) {
        loop {
            s.receive::<Repeat2>();
        }
    }

    #[macros::infer_session_type]
    fn D(mut s: Session) {
        loop {
            s.send(Repeat2);
        }
    }

    let a_role = Participant::new(Some(String::from("A")));
    let b_role = Participant::new(Some(String::from("B")));
    let c_role = Participant::new(Some(String::from("C")));
    let d_role = Participant::new(Some(String::from("D")));

    let a_mpst_local = get_rumpsteak_session_type_A().unwrap();
    let b_mpst_local = get_rumpsteak_session_type_B().unwrap();
    let c_mpst_local = get_rumpsteak_session_type_C().unwrap();
    let d_mpst_local = get_rumpsteak_session_type_D().unwrap();

    println!("A.MPSTLocalType: {}", a_mpst_local);
    println!("B.MPSTLocalType: {}", b_mpst_local);
    println!("C.MPSTLocalType: {}", c_mpst_local);
    println!("D.MPSTLocalType: {}", d_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(a_role, a_mpst_local), (b_role, b_mpst_local), (c_role, c_mpst_local), (d_role, d_mpst_local)])).unwrap());
}

#[test]
fn nested_recursion() {
    // Global type: RecX { A->B:Hi. RecY { B->A:Branch {1. X, 2. Y} } }
    // Local type A: RecX { Send<B, Hi, RecY { Branch<B, {1. X, 2. Y} } } }
    // Local type B: RecY { Receive<A, Hi, RecX { Select<A, {1. X, 2. Y} } } }

    struct Hi;
    enum Choice {
        RepeatX,
        RepeatY
    }

    impl Message for Hi {
        fn receive() -> Self {
            Hi
        }
    }

    impl Message for Choice {
        fn receive() -> Self {
            Choice::RepeatX
        }
    }

    let anon = Participant::anonymous();

    let ltA = MPSTLocalType::recX_with_id(
        Box::new(
            MPSTLocalType::send(
                anon.clone(),
                String::from("Hi"),
                MPSTLocalType::recX_with_id(
                    Box::new(
                        MPSTLocalType::Branch(
                            anon.clone(),
                            vec![
                                (String::from("RepeatX"), MPSTLocalType::x_with_id(1)),
                                (String::from("RepeatY"), MPSTLocalType::x_with_id(2))
                            ]
                        )
                    ),
                    2
                )
            )
        ),
        1
    );

    let ltB = MPSTLocalType::recX_with_id(
        Box::new(
            MPSTLocalType::receive(
                anon.clone(),
                String::from("Hi"),
                MPSTLocalType::recX_with_id(
                    Box::new(
                        MPSTLocalType::Select(
                            anon.clone(),
                            vec![
                                (String::from("RepeatX"), MPSTLocalType::x_with_id(1)),
                                (String::from("RepeatY"), MPSTLocalType::x_with_id(2))
                            ]
                        )
                    ),
                    2
                )
            )
        ),
        1
    );

    println!("A.MPSTLocalType: {}", ltA);
    println!("B.MPSTLocalType: {}", ltB);

    println!("{}", merge_locals(Parties::new(vec![(Participant::new(Some(String::from("A"))), ltA), (Participant::new(Some(String::from("B"))), ltB)])).unwrap());
}

#[test]
fn unsynchronised_recursion() {
    struct Hi;
    struct Hello;

    impl Message for Hi {
        fn receive() -> Self {
            Hi
        }
    }

    impl Message for Hello {
        fn receive() -> Self {
            Hello
        }
    }

    #[macros::infer_session_type]
    fn A(mut s: Session) {
        s.send(Hi);
        loop {
            s.receive::<Hello>();
            s.send(Hi);
        }
    }

    #[macros::infer_session_type]
    fn B(mut s: Session) {
        loop {
            s.receive::<Hi>();
            s.send(Hello);
        }
    }

    let a_role = Participant::new(Some(String::from("A")));
    let b_role = Participant::new(Some(String::from("B")));

    let a_mpst_local = get_rumpsteak_session_type_A().unwrap();
    let b_mpst_local = get_rumpsteak_session_type_B().unwrap();

    println!("A.MPSTLocalType: {}", a_mpst_local);
    println!("B.MPSTLocalType: {}", b_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(a_role, a_mpst_local), (b_role, b_mpst_local)])).unwrap());
}