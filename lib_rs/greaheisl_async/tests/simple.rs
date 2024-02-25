use greaheisl_async::{join2, sleep_at_most, AccessTiming, InstantMillis, MiniExecutor};
use std::rc::Rc;
use std::cell::RefCell;

async fn my_main_task(sys: impl AccessTiming, result: Rc<RefCell<Vec<i32>>>) {
    result.borrow_mut().push(-1);
    sleep_at_most(&sys,100).await;
    result.borrow_mut().push(-2);
    sleep_at_most(&sys,100).await;
    result.borrow_mut().push(-3);
    sleep_at_most(&sys,100).await;
    result.borrow_mut().push(-4);
    sleep_at_most(&sys,100).await; 
    join2(async {
        result.borrow_mut().push(-10);
        sleep_at_most(&sys,100).await;
        result.borrow_mut().push(-20);
        sleep_at_most(&sys,100).await;
    }, async {
        result.borrow_mut().push(-100);
        sleep_at_most(&sys,99).await;
        result.borrow_mut().push(-200);
        sleep_at_most(&sys,99).await;
    })
    .await;
}

#[test]
fn simple_test() {
    let mut time = InstantMillis::from_absolute(10);
    let executor_builder = MiniExecutor::new(time);
    let sys = executor_builder.scheduler().clone();
    let result = Rc::new(RefCell::new(Vec::new()));
    let mut executor = executor_builder.build(my_main_task(sys,result.clone()));
    result.borrow_mut().push(time.into_inner() as i32);
    time += 10;
    for _i in 0..6 {
        if let Some(max_wait) = executor.step(time,()) {
            time += max_wait;
            result.borrow_mut().push(time.into_inner() as i32);
        }
    }
    let result = result.borrow().clone();
    assert_eq!(result,vec![10, -1, 120, -2, 220, -3, 320, -4, 420, -10, -100, 519, -20, -200, 618])
}