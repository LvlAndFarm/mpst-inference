pub struct Session;

pub trait Message {
    fn receive() -> Self;
}

impl Session {
    pub fn new() -> Session {
        Session {}
    }

    pub fn send<T>(&mut self, _msg: T) {
        println!("Sending message");
    }

    pub fn receive<T: Message>(&mut self) -> T {
        println!("Receiving message");
        T::receive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn it_works() {
        let mut s = Session::new();
        s.send(Hello);
        s.receive::<Bye>();
    }
}
