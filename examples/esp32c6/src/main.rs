#![no_std]
#![no_main]

use log::{info};
use esp_backtrace as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    timer::OneShotTimer,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use bgt60trxx::{Radar, Variant, config::Config as RadarConfig};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

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
            .with_frequency(Rate::from_khz(100))
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

    let mut radar = Radar::new(Variant::BGT60TR13C, spi_device, rst, irq, delay2).await.unwrap();
    info!("Radar initialized!");

    let config = RadarConfig::default();

    info!("Configuring radar with: {}", config);

    radar.configure(config).await.unwrap();
    info!("Radar configured!");

    // TODO: Spawn some tasks
    let _ = spawner;

    radar.enable_test_mode().await.unwrap();
    radar.start().await.unwrap();
    info!("Radar frame generation started!");

    let mut buffer = [0u8; 192+4];
    let mut output = [0u16; 128];

    let mut test_word = 0x0001u16;
    let mut output_test = [0u16; 128];
    let mut error = false;

    loop {
        radar.get_fifo_data(&mut buffer, &mut output).await.unwrap();

        for i in 0..128 {
            output_test[i] = test_word;
            test_word = bgt60trxx::get_next_test_word(test_word);
        }

        // Check if the output matches the test word
        for i in 0..128 {
            if output[i] != output_test[i] {
                info!("Output mismatch at index {}: expected {}, got {}", i, output_test[i], output[i]);
                error = true;
            }
        }

        if error {
            info!("Mismatch in test data!");
            led_r.set_high();
            led_g.set_low();
            led_b.set_low();
        } else {
            info!("Frame correctly received!");
            led_r.set_low();
            led_g.set_high();
            led_b.set_low();
        }
    }
}
