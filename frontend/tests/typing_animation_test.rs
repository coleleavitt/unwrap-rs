#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_gloo_timeout_drop_cancels() {
    use gloo_timers::callback::Timeout;
    use std::cell::RefCell;
    use std::rc::Rc;

    let fired = Rc::new(RefCell::new(false));
    let fired_clone = fired.clone();

    let timeout = Timeout::new(10, move || {
        *fired_clone.borrow_mut() = true;
    });

    drop(timeout);

    assert!(!*fired.borrow(), "Timeout should not fire after drop");
}

#[wasm_bindgen_test]
fn test_gloo_interval_drop_cancels() {
    use gloo_timers::callback::Interval;
    use std::cell::RefCell;
    use std::rc::Rc;

    let count = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    let interval = Interval::new(10, move || {
        *count_clone.borrow_mut() += 1;
    });

    drop(interval);

    assert_eq!(*count.borrow(), 0, "Interval should not tick after drop");
}

#[wasm_bindgen_test]
fn test_option_timeout_replacement_cancels_previous() {
    use gloo_timers::callback::Timeout;
    use std::cell::RefCell;
    use std::rc::Rc;

    let first_fired = Rc::new(RefCell::new(false));
    let first_fired_clone = first_fired.clone();

    let first_timeout = Timeout::new(10, move || {
        *first_fired_clone.borrow_mut() = true;
    });

    let mut holder: Option<Timeout> = Some(first_timeout);
    holder = Some(Timeout::new(10, || {}));
    drop(holder);

    assert!(
        !*first_fired.borrow(),
        "First timeout should be cancelled when replaced"
    );
}

#[wasm_bindgen_test]
fn test_rc_refcell_timeout_pattern() {
    use gloo_timers::callback::Timeout;
    use std::cell::RefCell;
    use std::rc::Rc;

    let timer: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));
    let fired = Rc::new(RefCell::new(false));
    let fired_clone = fired.clone();

    *timer.borrow_mut() = Some(Timeout::new(10, move || {
        *fired_clone.borrow_mut() = true;
    }));

    assert!(timer.borrow().is_some(), "Timer should be stored");

    *timer.borrow_mut() = None;

    assert!(
        !*fired.borrow(),
        "Timer should be cancelled when set to None"
    );
}
