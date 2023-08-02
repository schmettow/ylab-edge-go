pub mod fake {
    use embassy_time::{Duration, Ticker, Instant};

    /* data */
    pub type Reading = (u16, u16, u16, u16);
    pub struct Result {
        pub time: Instant,
        pub reading: Reading
    }
    /* result channel */
    use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
    use embassy_sync::signal::Signal;
    pub static RESULT: Signal<ThreadModeRawMutex, Result> 
    = Signal::new();
    
    /* control channel */
    pub enum State {Ready, Record}
    pub static CONTROL: Signal<ThreadModeRawMutex, State> 
    = Signal::new();
    
    #[embassy_executor::task]
    pub async fn task(hz: u64) {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
                let reading: (u16, u16, u16, u16) = (0,0,0,0);
                let now: Instant = Instant::now();
                let result = Result{time: now, reading: reading};
                RESULT.signal(result);
                ticker.next()
                .await;
                };
            }
        }



    pub mod adc {
        use embassy_time::{Duration, Ticker, Instant};
        use embassy_rp::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29, ADC};
        use embassy_rp::adc::{Adc, Config, InterruptHandler};
        use embassy_rp::bind_interrupts;
        bind_interrupts!(struct Irqs {
            ADC_IRQ_FIFO => InterruptHandler;
        });

        /* data */
        pub struct SensorResult<R> { // <-- redundant
            pub time: Instant,
            pub reading: R,
        }
        pub type Reading = (u16, u16, u16, u16);
        pub struct Result {
            pub time: Instant,
            pub reading: Reading
        }
        /* result channel */
        use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
        use embassy_sync::signal::Signal;
        pub static RESULT: Signal<ThreadModeRawMutex, Result> 
        = Signal::new();
        
        /* control channel */
        pub enum State {Ready, Record}
        pub static CONTROL: Signal<ThreadModeRawMutex, State> 
        = Signal::new();
        //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;
        
        #[embassy_executor::task]
        pub async fn task(//mut adc: Adc<'static>, 
                        adc_contr: ADC,
                        mut adc_0: PIN_26,
                        mut adc_1: PIN_27,
                        mut adc_2: PIN_28,
                        mut adc_3: PIN_29,
                        hz: u64) {
            //let adc: Adc<'_> = Adc::new(p.ADC, Irqs, Config::default());
            let mut adc: Adc<'_> = Adc::new(adc_contr, Irqs, Config::default());
            let mut ticker = Ticker::every(Duration::from_hz(hz));
            //let _state: State = State::Ready;
            loop {
               /* match CONTROL.wait().await { 
                    State::Ready => {},
                    State::Record => {*/
                let mut reading: Reading = (0,0,0,0);
                reading.0 = adc.read(&mut adc_0).await;
                reading.1 = adc.read(&mut adc_1).await;
                reading.2 = adc.read(&mut adc_2).await;
                reading.3 = adc.read(&mut adc_3).await;
                let now: Instant = Instant::now();
                let result = Result{time: now, 
                                            reading: reading};
                RESULT.signal(result);
                ticker.next()
                .await;
                  //  },
                //}
            }
        }
    }

