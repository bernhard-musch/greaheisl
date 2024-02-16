mod timing {

    use crate::system::buttons::{ButtonFlags, SysButtonProcessor};
    use crate::system::SignalFlags;
    use alloc::rc::Rc;
    use ambassador::Delegate;
    use core::cell::{Cell, RefCell};
    use greaheisl_async::join2;
    use greaheisl_async::{
        ambassador_impl_AccessExecutorSignals, ambassador_impl_AccessTiming, AccessExecutorSignals,
        AccessTiming, DurationWrapper, MiniExecutor, MiniScheduler, Scheduler, Timer,
    };
    use greaheisl_async::{DurationMillis, InstantMillis};
    use greaheisl_button_processor::{
        AccessButtonSignal, AccessButtonState, ButtonEvent, ButtonProcessorOptions,
    };
    use std::time::Instant;

    fn instantmillis_from_duration(duration: std::time::Duration) -> InstantMillis {
        InstantMillis::from_absolute(duration.as_millis() as u32)
    }

    async fn timing_test_task_main(scheduler: impl Scheduler<SignalFlags>) {
        for k in 0..5 {
            println!("Counting: {}", k);
            Timer::new(&scheduler, 500).wait().await;
        }
        let stop_signal = Cell::new(false);
        join2(
            async {
                for i in 0..10 {
                    println!("Slow counting: {}", i);
                    Timer::new(&scheduler, 500).wait().await;
                }
                println!("Setting stop signal.");
                stop_signal.set(true);
            },
            async {
                let mut counter = 0;
                while stop_signal.get() != true {
                    println!("Fast counting: {}", counter);
                    counter += 1;
                    Timer::new(&scheduler, 230).wait().await;
                }
                println!("Received stop signal.");
            },
        )
        .await;
        for k in 0..5 {
            println!("Counting: {}", k);
            Timer::new(&scheduler, 500).wait().await;
        }
    }

    #[test]
    fn timing_test() {
        let start_instant = Instant::now();
        let instant = instantmillis_from_duration(start_instant.elapsed());
        let executor = MiniExecutor::new(instant);
        let task = timing_test_task_main(executor.scheduler().clone());
        let mut executor = executor.build(task);
        loop {
            let instant = instantmillis_from_duration(start_instant.elapsed());
            let signal_flags = crate::system::SignalFlags::none();
            let Some(delay_millis) = executor.step(instant, signal_flags) else {
                break;
            };
            println!("Waiting {} milli seconds.", delay_millis);
            std::thread::sleep(std::time::Duration::from_millis(delay_millis as u64));
        }
    }

    #[derive(Clone, Delegate)]
    #[delegate(AccessTiming, target = "scheduler")]
    #[delegate(AccessExecutorSignals<SignalFlags>,target = "scheduler")]
    struct ButtonTestSys {
        scheduler: Rc<RefCell<MiniScheduler<SignalFlags>>>,
        buttons: Rc<Cell<ButtonFlags>>,
    }

    impl AccessButtonState for ButtonTestSys {
        type ButtonFlags = ButtonFlags;
        fn get_button_flags(&self) -> crate::system::buttons::ButtonFlags {
            self.buttons.get()
        }
    }

    impl AccessButtonSignal for ButtonTestSys {
        fn is_button_signal(&self) -> bool {
            self.get_executor_signals().contains(SignalFlags::Button)
        }
    }

    /* This does not work due to lifetime issues
    async fn test_inner(stuff: &usize) {
        println!("got {}",stuff);
    }

    use core::future::Future;
    async fn test_outer<'a,F: Future<Output = ()> + 'a>(f: impl Fn(&'a usize)->F) {
        let a = 0usize;
        f(&a).await;
    }

    async fn button_test_task_main(sys: impl Scheduler+AccessButtonState  + 'static) {
        test_outer(|b| test_inner(b)).await
    }
    */

    async fn button_test_task_main(
        sys: impl Scheduler<SignalFlags>
            + AccessButtonState<ButtonFlags = ButtonFlags>
            + AccessButtonSignal
            + 'static,
    ) {
        let bp = SysButtonProcessor::new(ButtonProcessorOptions::default());
        bp.run(&sys, async {
            loop {
                let ev = bp.event();
                println!("Got event {:?} state {:?}", ev, bp.state());
                let timer = Timer::new(&sys, 1000);
                while timer.yield_if_time_left().await {
                    let ButtonEvent::None = bp.event() else {
                        break;
                    };
                }
            }
        })
        .await;
    }

    #[test]
    fn button_test() {
        let button_sequence: Vec<ButtonFlags> = vec![
            ButtonFlags::none(),
            ButtonFlags::none(),
            ButtonFlags::Escape,
            ButtonFlags::Escape,
            ButtonFlags::Escape,
            ButtonFlags::Escape,
            ButtonFlags::none(),
            ButtonFlags::none(),
            ButtonFlags::Enter,
            ButtonFlags::none(),
            ButtonFlags::Prev,
            ButtonFlags::Next,
            ButtonFlags::none(),
            ButtonFlags::Prev,
            ButtonFlags::Prev | ButtonFlags::Next,
            ButtonFlags::Next,
            ButtonFlags::none(),
        ];
        let mut fake_time = InstantMillis::from_absolute(0);
        let executor = MiniExecutor::new(fake_time);
        let buttons = Rc::new(Cell::new(ButtonFlags::none()));
        let sys = ButtonTestSys {
            scheduler: executor.scheduler().clone(),
            buttons: buttons.clone(),
        };
        let task = button_test_task_main(sys);
        let mut executor = executor.build(task);
        const MAX_WAIT: DurationMillis = 12345;
        for elem in button_sequence {
            let signal_flags = crate::system::SignalFlags::Button;
            buttons.set(elem);
            let timestep = {
                if let Some(delay_millis) = executor.step(fake_time, signal_flags) {
                    println!("Waiting {} milli seconds.", delay_millis);
                    delay_millis.min(MAX_WAIT)
                } else {
                    MAX_WAIT
                }
            };
            fake_time += timestep;
        }
    }
}
