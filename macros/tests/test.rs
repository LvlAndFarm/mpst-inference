use macros::infer_session_type;
use session::*;

struct Hello;
struct Olleh;
struct Bye;
struct Query;
struct Response;

enum Status {
    Healthy,
    Sick
}

impl Message for Hello {
    fn receive() -> Self {
        Hello
    }
}

impl Message for Olleh {
    fn receive() -> Self {
        Olleh
    }
}

impl Message for Bye {
    fn receive() -> Self {
        Bye
    }
}

impl Message for Response {
    fn receive() -> Self {
        Response
    }
}

impl Message for Status {
    fn receive() -> Self {
        Status::Healthy
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
    s.send(Bye);
}

fn _ignore(_l: Olleh, _ll: Olleh) {
}

#[infer_session_type]
fn example_internal_choice(mut s: Session) {
    s.send(Hello);
    s.receive::<Olleh>();
    println!("Hello world");
    
    let mut i = 0;
    while i < 10 {
        s.send(Query);
        s.receive::<Response>();
        i+=1;
    }
    s.send(Bye);
}

#[infer_session_type]
fn example_conflicting_choice(mut s: Session) {
    s.send(Hello);
    s.receive::<Olleh>();
    println!("Hello world");
    
    let mut i = 0;
    while i < 10 {
        s.send(Query);
        s.receive::<Response>();
        i+=1;
    }
    s.send(Query);
}

#[infer_session_type]
fn example_func_arg_calls(mut s: Session) {
    s.send(Hello);
    ({
        s.send(Hello);
        _ignore
    })(s.receive::<Olleh>(), s.receive::<Olleh>());
    println!("Hello world");
    
    let mut i = 0;
    while i < 10 {
        s.send(Query);
        s.receive::<Response>();
        i+=1;
    }
    s.send(Bye);
}

#[infer_session_type]
fn example_external_choice(mut s: Session) {
    s.send(Hello);
    s.receive::<Olleh>();
    println!("Hello world");
    
    match s.branch::<Status>() {
        Status::Healthy => {
            let mut i = 0;
            while i < 10 {
                s.send(Query);
                s.receive::<Response>();
                i+=1;
            }
            s.send(Bye);
        },
        Status::Sick => {
            s.send(Bye);
        }
    
    }
}

#[test]
fn it_works() {
    println!("{}", get_session_type_example_external_choice());
    println!("{}", get_mpst_session_type_example_external_choice().unwrap());
    // let result = add(2, 2);
    // assert_eq!(result, 4);
}