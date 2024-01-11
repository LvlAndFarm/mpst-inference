use macros::infer_session_type;
use session::*;

struct Hello;
struct Bye;

impl Message for Hello {
    fn receive() -> Self {
        Hello
    }
}

impl Message for Bye {
    fn receive() -> Self {
        Bye
    }
}

#[infer_session_type]
fn example(mut s: Session) {
    s.send(Hello);
    s.receive::<Bye>();
    println!("Hello world");
    while true {
        s.send(Hello);
        s.receive::<Bye>();
    }
    s.receive::<Bye>();
}

#[test]
fn it_works() {
    print_session_type()
    // let result = add(2, 2);
    // assert_eq!(result, 4);
}