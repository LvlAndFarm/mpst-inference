use macros::infer_session_type;
use session::*;

struct Hello;
struct Olleh;
struct Bye;
struct Query;
struct Response;

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

fn _ignore(_l: Hello, _ll: Hello) {
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

#[test]
fn it_works() {
    print_session_type_example_func_arg_calls()
    // let result = add(2, 2);
    // assert_eq!(result, 4);
}