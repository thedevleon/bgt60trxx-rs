#![no_std]
#![no_main]

use defmt::info;
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
use panic_rtt_target as _;
use bgt60trxx::{Radar, Variant, config::Config as RadarConfig};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    let delay1 = OneShotTimer::new(timer0.alarm1).into_async();
    let delay2 = OneShotTimer::new(timer0.alarm2).into_async();

    info!("Embassy initialized!");

    let sclk = peripherals.GPIO0;
    let miso = peripherals.GPIO2;
    let mosi = peripherals.GPIO4;
    let cs = peripherals.GPIO5;
    let rst = peripherals.GPIO6;
    let irq = peripherals.GPIO7;

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

    let mut radar = Radar::new(Variant::BGT60TR13C, spi_device, rst, irq, delay2).await.unwrap();
    info!("Radar initialized!");

    let config = RadarConfig::default();
    radar.configure(config).await.unwrap();
    info!("Radar configured!");

    // TODO: Spawn some tasks
    let _ = spawner;

    radar.enable_test_mode().await.unwrap();
    radar.start().await.unwrap();

    let mut buffer = [0u8; 9216];
    let mut test_word = 0x0001u16;

    loop {
        radar.get_fifo_data(&mut buffer).await.unwrap();

        // TODO map from u8 to u12 to u16
        // TODO verify test pattern

        test_word = bgt60trxx::get_next_test_word(test_word);

    }
}
