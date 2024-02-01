#[cfg(test)]

use session::{session_type::{MPSTLocalType, Participant}, Session, Message};
use inference::merging::*;

#[test]
fn travel_agency() {
    struct Order {
        max_distance_km: u32
    }
    struct Quote(u32);
    enum Response { 
        Accept(bool),
        Reject(i32)
    }
    struct Address(String);
    struct Date(u32);

    let x = Date(3);

    impl Message for Order {
        fn receive() -> Self {
            Order {
                max_distance_km: 0
            }
        }
    }

    impl Message for Quote {
        fn receive() -> Self {
            Quote(0)
        }
    }

    impl Message for Response {
        fn receive() -> Self {
            Response::Accept(false)
        }
    }

    impl Message for Address {
        fn receive() -> Self {
            Address("".into())
        }
    }

    impl Message for Date {
        fn receive() -> Self {
            Date(0)
        }
    }

    #[macros::infer_session_type]
    fn client(mut s: Session) {
        let max_distance_km = 30;
        s.send(Order {
            max_distance_km
        });
        let quote = s.receive::<Quote>();
        
        if quote.0 > 100 {
            s.send(Response::Reject(-1));
        } else {
            s.send(Response::Accept(true));
            s.send(Address("123 Fake Street".into()));
            let date = s.receive::<Date>();
            println!("Booked order: {}", date.0);
        }
    }

    #[macros::infer_session_type]
    fn agency(mut s: Session) {
        let order = s.receive::<Order>();
        let distance = order.max_distance_km;
        s.send(Quote(distance * 5));
        match s.branch::<Response>() {
            Response::Accept(_) => {
                s.receive::<Address>();
                s.send(Date(5));
            }
            Response::Reject(_) => {
                println!("Rejected order");
            }
        }
    }

    let client_role = Participant::new(Some(String::from("C")));
    let agency_role = Participant::new(Some(String::from("A")));

    let client_mpst_local = get_mpst_session_type_client().unwrap();
    let agency_mpst_local = get_mpst_session_type_agency().unwrap();

    println!("Client.MPSTLocalType: {}", client_mpst_local);
    println!("Agency.MPSTLocalType: {}", agency_mpst_local);

    println!("{}", merge_locals(Parties::new(vec![(client_role, client_mpst_local), (agency_role, agency_mpst_local)])).unwrap());
}