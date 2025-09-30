
extern "C" {
    fn c_foo(c: *mut i32);
    fn c_print_answer_to_universe();
}

fn foo() {
    let mut c = Box::new(42);
    unsafe {
        c_foo(c.as_mut() as *mut i32);
        c_print_answer_to_universe();
    }
}

fn main() {
    println!("Hello, world!");
    foo();
    unsafe {
        c_print_answer_to_universe();
    }
}
