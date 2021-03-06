//! examples/lock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[resources]
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtic::pend(Interrupt::GPIOA);

        init::LateResources {}
    }

    // when omitted priority is assumed to be `1`
    #[task(binds = GPIOA, resources = [shared])]
    fn gpioa(mut c: gpioa::Context) {
        hprintln!("A").unwrap();

        // the lower priority task requires a critical section to access the data
        c.resources.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // GPIOB will *not* run right now due to the critical section
            rtic::pend(Interrupt::GPIOB);

            hprintln!("B - shared = {}", *shared).unwrap();

            // GPIOC does not contend for `shared` so it's allowed to run now
            rtic::pend(Interrupt::GPIOC);
        });

        // critical section is over: GPIOB can now start

        hprintln!("E").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = GPIOB, priority = 2, resources = [shared])]
    fn gpiob(mut c: gpiob::Context) {
        // the higher priority task does still need a critical section
        let shared = c.resources.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        hprintln!("D - shared = {}", shared).unwrap();
    }

    #[task(binds = GPIOC, priority = 3)]
    fn gpioc(_: gpioc::Context) {
        hprintln!("C").unwrap();
    }
}
