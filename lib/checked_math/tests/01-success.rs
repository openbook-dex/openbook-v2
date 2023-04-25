use checked_math::{checked_math, checked_math_or_panic};
use std::cell::RefCell;
use std::rc::Rc;

fn f() -> u8 {
    3u8
}

struct S {}
impl S {
    fn m(&self) -> u8 {
        2u8
    }
}

fn main() {
    let num = 2u8;

    let result = checked_math!{ (num + (2u8 / 10)) * 5 };
    assert!(result == Some(10));

    let result = checked_math!{ ((num.pow(20) << 20) + 255) + 2u8 * 2u8 };
    assert!(result == None);

    let result = checked_math!{ -std::i8::MIN };
    assert!(result == None);

    let result = checked_math!{ 12u8 + 6u8 / 3 };
    assert!(result == Some(14));

    let result = checked_math!{ 12u8 + 6u8 / f() };
    assert!(result == Some(14));

    let result = checked_math!{ 12u8 + 6u8 / num };
    assert!(result == Some(15));

    let s = S{};
    let result = checked_math!{ 12u8 + s.m() };
    assert!(result == Some(14));

    let r = checked_math_or_panic!(num + 4u8);
    assert_eq!(r, 6);

    let mut m = 2u8;
    checked_math_or_panic!(m += 4);
    assert_eq!(m, 6);

    let g = Rc::new(RefCell::new(0u8));
    let single_eval_test = || -> Rc<RefCell<u8>> {
        *g.borrow_mut() += 1;
        g.clone()
    };

    *single_eval_test().borrow_mut() += 10;
    assert_eq!(*g.borrow(), 11);

    // I don't get why this passes:
    // The macro should call the left hand side expression multiple times?
    checked_math_or_panic!(*single_eval_test().borrow_mut() += 10);
    assert_eq!(*g.borrow(), 22);

    eprintln!("Ignore STDERR messages if the test passes: the panics were captured");
    assert!(std::panic::catch_unwind(|| {
        let mut m = 2u8;
        checked_math_or_panic!(m /= 0);
        assert_eq!(m, 0); // unreached
    }).is_err());
}
