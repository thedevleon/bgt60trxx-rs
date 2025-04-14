#![no_std]
#![no_main]

use bgt60trxx::{config::Config as RadarConfig, Radar, Variant};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
    timer::OneShotTimer,
};
use log::info;
use ndarray::prelude::*;
extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    let delay1 = OneShotTimer::new(timer0.alarm1).into_async();
    let delay2 = OneShotTimer::new(timer0.alarm2).into_async();

    info!("Embassy initialized!");

    // Specific to KIT CSK BGT60TR13C
    let led_r = peripherals.GPIO0;
    let led_g = peripherals.GPIO1;
    let led_b = peripherals.GPIO2;
    let ldo_en = peripherals.GPIO15;

    let mut led_r = Output::new(led_r, Level::Low, OutputConfig::default());
    let mut led_g = Output::new(led_g, Level::Low, OutputConfig::default());
    let mut led_b = Output::new(led_b, Level::Low, OutputConfig::default());
    let mut ldo_en = Output::new(ldo_en, Level::Low, OutputConfig::default());

    let sclk = peripherals.GPIO19;
    let miso = peripherals.GPIO21;
    let mosi = peripherals.GPIO20;
    let cs = peripherals.GPIO9;
    let rst = peripherals.GPIO11;
    let irq = peripherals.GPIO10;

    let cs = Output::new(cs, Level::High, OutputConfig::default());
    let rst = Output::new(rst, Level::High, OutputConfig::default());
    let irq = Input::new(irq, InputConfig::default());

    let dma_channel = peripherals.DMA_CH0;
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(32)) // 32 MHz seems to be around the max with the 
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let spi_device = ExclusiveDevice::new(spi_bus, cs, delay1).unwrap();

    // Turn on the LDO
    ldo_en.set_high();
    Timer::after(Duration::from_millis(500)).await; // Wait for LDO to stabilize

    let mut radar = Radar::new(Variant::BGT60TR13C, spi_device, rst, irq, delay2)
        .await
        .unwrap();
    info!("Radar initialized!");

    // radar_low_framerate_single_antenna_config.json
    let config = RadarConfig::new(
        1,
        1,
        31,
        60,
        61020099000,
        61479903000,
        16,
        128,
        6.99625e-05,
        0.100057,
        2352941,
        [
            0x11e8270, 0x3088210, 0x9e967fd, 0xb0805b4, 0xd1027ff, 0xf010700, 0x11000000,
            0x13000000, 0x15000000, 0x17000be0, 0x19000000, 0x1b000000, 0x1d000000, 0x1f000b60,
            0x21103c51, 0x231ff41f, 0x25006f7b, 0x2d000490, 0x3b000480, 0x49000480, 0x57000480,
            0x5911be0e, 0x5b677c0a, 0x5d00f000, 0x5f787e1e, 0x61f5208a, 0x630000a4, 0x65000252,
            0x67000080, 0x69000000, 0x6b000000, 0x6d000000, 0x6f093910, 0x7f000100, 0x8f000100,
            0x9f000100, 0xad000000, 0xb7000000,
        ],
    );

    info!("Configuring radar with: {}", config);

    radar.configure(config).await.unwrap();
    info!("Radar configured!");

    // TODO: Spawn some tasks
    let _ = spawner;

    radar.enable_test_mode().await.unwrap();
    radar.start().await.unwrap();
    info!("Radar frame generation started!");

    let mut test_word = 0x0001u16;
    let mut error = false;

    loop {
        let frames = radar.get_frames().await.unwrap();
        // info!(
        //     "Frame received, shape: {:?}, content: {:?}",
        //     frame.shape(),
        //     frame
        // );

        // we only care about the first antenna (since the test mode only replaces the first antenna)
        let rx0_output = frames.slice(s![0, .., ..]);

        // go through each chirp
        for chirp in rx0_output.outer_iter() {
            // go through each sample in the chirp
            for sample in chirp.iter() {
                if *sample != test_word {
                    // info!("Output mismatch at index {}: expected {}, got {}", i, test_word, sample);
                    error = true;
                }

                test_word = bgt60trxx::get_next_test_word(test_word);
            }
        }

        if error {
            info!("Mismatch in test data!");
            led_r.set_high();
            led_g.set_low();
            led_b.set_low();
        } else {
            info!("Frame verified!");
            led_r.set_low();
            led_g.set_high();
            led_b.set_low();
        }
    }
}
