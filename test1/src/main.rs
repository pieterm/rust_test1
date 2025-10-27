#![no_std]
#![no_main]

use defmt::Format;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println as _;

use embedded_graphics::{
    image::Image, pixelcolor::Rgb565, prelude::*
};
 
use mipidsi::{
    interface::SpiInterface, models::ST7789, options::{ColorInversion, Orientation, Rotation}
};

use esp_hal::{
    gpio::{
        Level,
	Input,
	InputConfig,
        Output,
        OutputConfig
    },
    spi::{
        Mode,
        master::{
            Spi,
            Config,
        },
    },
    timer::timg::TimerGroup,
    time::Rate,
    delay::Delay,
};

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_sync::{
    watch::Watch,
    blocking_mutex::raw::CriticalSectionRawMutex
};
use tinybmp::Bmp;

const TFT_WIDTH: u16 = 135;
const TFT_HEIGHT: u16 = 240;

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
    // Borrow the needed Peripherals and set the pins
    let config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let button1_pin = Input::new(peripherals.GPIO35, InputConfig::default());
    let button2_pin = Input::new(peripherals.GPIO0, InputConfig::default());

    let spi = peripherals.SPI2;
    let mosi = peripherals.GPIO19;
    let sclk = peripherals.GPIO18;
    let pin_chip_select   = peripherals.GPIO5;
    let pin_spi_datacommand   = peripherals.GPIO16;
    let pin_reset = peripherals.GPIO23;
    let pin_backlight   = peripherals.GPIO4;

    defmt::info!("init peripherals completed...");
    // Initialize the SPI interface
    let spi_bus = Spi::new(
            spi,
            Config::default()
                .with_frequency(Rate::from_mhz(26))
                .with_mode(Mode::_0)
            ).unwrap()
            .with_sck(sclk)
            .with_mosi(mosi);

    let config = OutputConfig::default();
    let cs_output = Output::new(pin_chip_select, Level::High, config);
    let dc_output = Output::new(pin_spi_datacommand, Level::Low, config);
    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, cs_output).unwrap();

    static mut SPI_BUFFER: [u8; 512] = [0; 512];
    let di = unsafe {
        SpiInterface<'static, ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, embedded_hal_bus::spi::NoDelay>, Output<'static>> = SpiInterface::new(
            spi_device,    
            dc_output,
            &mut SPI_BUFFER,
        )
    };

    let mut delay = Delay::new();
    let rst_output = Output::new(pin_reset, Level::High, config);

    let mut display: mipidsi::Display<SpiInterface<'_, ExclusiveDevice<Spi<'_, esp_hal::Blocking>, Output<'_>, embedded_hal_bus::spi::NoDelay>, Output<'_>>, ST7789, Output<'_>> = mipidsi::Builder::new(ST7789, di)
        .reset_pin(rst_output)
        .display_size(TFT_WIDTH as u16, TFT_HEIGHT as u16)
        .display_offset(52, 40)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay).unwrap();

    
    // Configure the backlight pin (TFT_BL) to output and set the pin on HIGH
    let mut backlight_output = Output::new(pin_backlight, Level::Low, config);
    backlight_output.set_high();

    defmt::info!("Init display complete");

    display.clear(Rgb565::BLACK).unwrap();

    spawner.spawn(print_button1_state_task()).ok();
    spawner.spawn(read_button1_task(button1_pin)).ok();
    spawner.spawn(print_button2_state_task()).ok();
    spawner.spawn(read_button2_task(button2_pin)).ok();
    spawner.spawn(draw_display_task(display)).ok();


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

#[embassy_executor::task]
async fn draw_display_task(mut display: mipidsi::Display<
        SpiInterface<'static, ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, embedded_hal_bus::spi::NoDelay>, Output<'static>>,
        ST7789,
        Output<'static>,
    >) {
    defmt::info!("Start draw_display_task");

    let logo = Bmp::from_slice(include_bytes!("../assets/pieter.bmp")).unwrap();
    
    loop{    
        let display_center = display.bounding_box().center().x_axis();
        let logo_center = logo.bounding_box().center().x_axis();
        let logo_position = display_center - logo_center;
        let image = Image::new(&logo, logo_position);
        image.draw(&mut display).unwrap();
     
        Timer::after(Duration::from_millis(3000)).await;
    }
}
