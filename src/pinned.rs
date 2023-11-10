use std::marker::PhantomPinned;
use std::pin::Pin;

#[derive(Debug)]
pub struct Test {
    a: String,
    b: *const String,
}

impl Test {
    fn new(txt: &str) -> Self {
        Self {
            a: String::from(txt),
            b: std::ptr::null(),
        }
    }

    fn init(&mut self) {
        let self_ref: *const String = &self.a;

        self.b = self_ref;
    }

    fn a(&self) -> &str {
        &self.a
    }

    fn b(&self) -> &str {
        unsafe { &(*self.b) }
    }
}

pub fn pin_test_simple() {
    let mut test1 = Test::new("test1");
    test1.init();
    let mut test2 = Test::new("test2");
    test2.init();

    println!(
        "Test1: {:p} a: {}, b pointer: {:p}, b: {}",
        &test1,
        test1.a(),
        test1.b,
        test1.b()
    );
    println!(
        "Test2: {:p} a: {}, b pointer: {:p}, b: {}",
        &test2,
        test2.a(),
        test2.b,
        test2.b()
    );

    std::mem::swap(&mut test1, &mut test2);

    println!("After swap");

    println!(
        "Test1: {:p} a: {}, b pointer: {:p}, b: {}",
        &test1,
        test1.a(),
        test1.b,
        test1.b()
    );
    println!(
        "Test2: {:p} a: {}, b pointer: {:p}, b: {}",
        &test2,
        test2.a(),
        test2.b,
        test2.b()
    );
}

#[derive(Debug)]
struct TestPin {
    a: String,
    b: *const String,
    _pin: PhantomPinned,
}

impl TestPin {
    fn new(txt: &str) -> Self {
        Self {
            a: String::from(txt),
            b: std::ptr::null(),
            _pin: PhantomPinned,
        }
    }

    fn init(self: Pin<&mut Self>) {
        let self_ref: *const String = &self.a;

        let this = unsafe { self.get_unchecked_mut() };
        this.b = self_ref;
    }

    fn a(self: Pin<&Self>) -> &str {
        &self.get_ref().a
    }

    fn b(self: Pin<&Self>) -> &str {
        unsafe { &(*self.b) }
    }
}

pub fn pin_test() {
    let mut test1 = TestPin::new("test1");
    let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
    TestPin::init(test1.as_mut());

    let mut test2 = TestPin::new("test2");
    let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    TestPin::init(test2.as_mut());

    println!(
        "Test1: {:p} a: {}, b pointer: {:p}, b: {}",
        &test1,
        TestPin::a(test1.as_ref()),
        test1.b,
        TestPin::b(test1.as_ref()),
    );

    println!(
        "Test2: {:p} a: {}, b pointer: {:p}, b: {}",
        &test2,
        TestPin::a(test2.as_ref()),
        test2.b,
        TestPin::b(test2.as_ref()),
    );

    //std::mem::swap(test1.get_mut(), test2.get_mut());
}
