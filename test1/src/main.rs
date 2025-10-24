#![no_std]
#![no_main]

use defmt::Format;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println as _;
use esp_hal::{
    timer::timg::TimerGroup,
    gpio::{Input, InputConfig}
};
use embassy_sync::{
    watch::Watch,
    blocking_mutex::raw::CriticalSectionRawMutex
};

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Format, Clone)]
enum ButtonState {
    Pressed,
    Released,
}

static BUTTON1_WATCH: Watch<CriticalSectionRawMutex, ButtonState, 2> = Watch::new();
static BUTTON2_WATCH: Watch<CriticalSectionRawMutex, ButtonState, 2> = Watch::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let button1_pin = Input::new(peripherals.GPIO35, InputConfig::default());
    let button2_pin = Input::new(peripherals.GPIO0, InputConfig::default());
    // let second_button_pin = Input::new(peripherals.GPIO0, InputConfig::default());

    spawner.spawn(print_button1_state_task()).ok();
    spawner.spawn(read_button1_task(button1_pin)).ok();
    spawner.spawn(print_button2_state_task()).ok();
    spawner.spawn(read_button2_task(button2_pin)).ok();
    // spawner.spawn(another_button_watching_task()).ok();
    // spawner.spawn(another_button_publishing_task(second_button_pin)).ok();


    loop {
        defmt::info!("main loop!");
        // you could do stuff here, but for now we keep this empty
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[embassy_executor::task]
async fn print_button1_state_task() {
    defmt::info!("start print_button1_state_task");
    let button_watch_receiver_result = BUTTON1_WATCH.receiver();
    // let button_watch_receiver_result = BUTTON_PUB_SUB.subscriber();
    match button_watch_receiver_result {
        // Ok(mut button_state_receiver) => {
        //     loop {
        //         let button_state = button_state_receiver.next_message().await;
        //         defmt::info!("button state: {:?}", button_state);
        //     }
        // }
        // Err(e) => {defmt::error!("no extra watchers available: {:?}", e)}
        Some(mut button_watch_receiver) => {
            loop {
                let button_state = button_watch_receiver.changed().await;
                defmt::info!("button state: {:?}", button_state);
            }
        }
        None => { defmt::error!("no extra watchers available!") }
    }
}

#[embassy_executor::task]
async fn print_button2_state_task() {
    defmt::info!("start print_button2_state_task");
    let button_watch_receiver_result = BUTTON2_WATCH.receiver();
    // let button_watch_receiver_result = BUTTON_PUB_SUB.subscriber();
    match button_watch_receiver_result {
        // Ok(mut button_state_receiver) => {
        //     loop {
        //         let button_state = button_state_receiver.next_message().await;
        //         defmt::info!("button state: {:?}", button_state);
        //     }
        // }
        // Err(e) => {defmt::error!("no extra watchers available: {:?}", e)}
        Some(mut button_watch_receiver) => {
            loop {
                let button_state = button_watch_receiver.changed().await;
                defmt::info!("button state: {:?}", button_state);
            }
        }
        None => { defmt::error!("no extra watchers available!") }
    }
}

#[embassy_executor::task]
async fn read_button1_task(mut button: Input<'static>){
    defmt::info!("start read_button1_task");
    let sender = BUTTON1_WATCH.sender();
    // let publisher = BUTTON_PUB_SUB.publisher().unwrap();
    loop {
        button.wait_for_falling_edge().await;
        sender.send(ButtonState::Pressed);
        // publisher.publish(ButtonState::Pressed).await;
        Timer::after(Duration::from_millis(5)).await; //debounce time
        button.wait_for_rising_edge().await;
        sender.send(ButtonState::Released);
        // publisher.publish(ButtonState::Released).await;
        Timer::after(Duration::from_millis(5)).await; //debounce time
    }
}

#[embassy_executor::task]
async fn read_button2_task(mut button: Input<'static>){
    defmt::info!("start read_button2_task");
    let sender = BUTTON2_WATCH.sender();
    // let publisher = BUTTON_PUB_SUB.publisher().unwrap();
    loop {
        button.wait_for_falling_edge().await;
        sender.send(ButtonState::Pressed);
        // publisher.publish(ButtonState::Pressed).await;
        Timer::after(Duration::from_millis(5)).await; //debounce time
        button.wait_for_rising_edge().await;
        sender.send(ButtonState::Released);
        // publisher.publish(ButtonState::Released).await;
        Timer::after(Duration::from_millis(5)).await; //debounce time
    }
}


// // optional: see what happens when you want to use multiple receivers for the watch.

// #[embassy_executor::task]
// async fn another_button_watching_task() {
//     defmt::info!("start another_button_watching_task");
//     let button_watch_receiver_result = BUTTON_WATCH.receiver();

//     match button_watch_receiver_result {
//         Some(mut button_watch_receiver) => {
//             loop {
//                 let button_state = button_watch_receiver.changed().await;
//                 defmt::info!("button state from another watching task: {:?}", button_state);
//             }
//         }
//         None => {defmt::error!("no extra watchers available!")}
//     }
// }


// (Optional and advanced)
// Create another task that also produces data. So, 2 producers and 2 consumers. Maybe you need another
// async datatype: https://docs.embassy.dev/embassy-sync/git/default/index.html

// use embassy_sync::pubsub::PubSubChannel;
// static BUTTON_PUB_SUB: PubSubChannel<CriticalSectionRawMutex, ButtonState, 1, 2, 2> = PubSubChannel::new();
//
// #[embassy_executor::task]
// async fn another_button_publishing_task(mut button: Input<'static>){
//     defmt::info!("start another_button_publishing_task");
//     let publisher = BUTTON_PUB_SUB.publisher().unwrap();
//     loop {
//         button.wait_for_falling_edge().await;
//         publisher.publish(ButtonState::Pressed).await;
//         Timer::after(Duration::from_millis(5)).await; //debounce time
//         button.wait_for_rising_edge().await;
//         publisher.publish(ButtonState::Released).await;
//         Timer::after(Duration::from_millis(5)).await; //debounce time
//     }
// }
//
// #[embassy_executor::task]
// async fn yet_another_button_watching_task() {
//     defmt::info!("start yet_another_button_watching_task");
//     let mut consumer = BUTTON_PUB_SUB.subscriber().unwrap();
//
//     loop {
//         let button_state = consumer.next_message().await;
//         defmt::info!("button state from yet another watching task: {:?}", button_state);
//     }
// }
