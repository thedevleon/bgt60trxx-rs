# `bgt60trxx`

[![Crate](https://img.shields.io/crates/v/bgt60trxx.svg)](https://crates.io/crates/bgt60trxx)
[![API](https://docs.rs/bgt60trxx/badge.svg)](https://docs.rs/bgt60trxx)

An async and no_std rust library to interface via SPI with the XENSIV™ BGT60TRxx 60 GHz FMCW radar sensors from Infineon.

## Supported Sensors
- BGT60TR13C
- BGT60UTR11AIP

## What works
- Reading and writing registers
- Resetting hardware, software, FIFO and fsm
- Parsing GSR0 register and returning matching errors
- Configuring the radar
- Burst reading FIFO
- Test mode and test word generation
- Converting the raw FIFO buffer into a correctly-shaped ndarray (requires `alloc` feature)

## Features
- `alloc`: enables `get_frames` method which returns FIFO data in a dynamically allocated 3D ndarray in the shape of `[rx_antenna, chirp, adc_sample]`
- `debug`: prints some debugging information via `log`


## Basic Usage
```rust
use bgt60trxx::{Radar, Variant, config::Config as RadarConfig};

// let sclk = ...
// let miso = ...
// let mosi = ...
// let cs = ...
// let rst = ...
// let irq = ...

// let spi_bus = ...
// let spi_device = ... (see embedded-hal-bus, e.g. ExclusiveDevice)
// let delay = ...

// if you want to use ndarray, alloc is required
// also make sure to enable the alloc feature of bgt60trxx-rs
extern crate alloc;

let mut radar = Radar::new(Variant::BGT60TR13C, spi_device, rst, irq, delay).await.unwrap();
info!("Radar initialized!");

let config = RadarConfig::default();
info!("Configuring radar with: {}", config);

radar.configure(config).await.unwrap();
info!("Radar configured!");

radar.start().await.unwrap();
info!("Radar frame generation started!");

loop {
    let frames = radar.get_frames(&mut buffer, &mut output).await.unwrap();
    // TODO: Post-processing of ADC data (adc -> hanning window -> range fft -> doppler fft -> ...) 
}
```

## Generating a new config
To generate a new config, use the below JSON template (taken from <https://github.com/Infineon/sensor-xensiv-bgt60trxx>), adjust it accordingly, and run it through bgt60-configurator-cli:
`./bgt60-configurator-cli -c settings.json -o settings.h`

```json
{
    "device_config": {
        "fmcw_single_shape": {
            "rx_antennas": [3], 
            "tx_antennas": [1], 
            "tx_power_level": 31, 
            "if_gain_dB": 60, 
            "lower_frequency_Hz": 61020098000, 
            "upper_frequency_Hz": 61479902000, 
            "num_chirps_per_frame": 32, 
            "num_samples_per_chirp": 128, 
            "chirp_repetition_time_s": 7e-05, 
            "frame_repetition_time_s": 5e-3, 
            "sample_rate_Hz": 2330000
        }
    }
}
```

This will generate a C header file with defines and a list of registers, which you can use to construct a config struct, see `Config::high_framerate_preset()`.

Note: The `bgt60-configurator-cli` has a bug where `XENSIV_BGT60TRXX_CONF_NUM_RX_ANTENNAS` is fixed to 1, even when the JSON config says 3.
Make sure to keep `rx_antennas` as 3 if that is the case.


## Modules

### BGT60TR13C
There are a couple of ready-made modules with the BGT60TR13C that include all required supporting components:
- KITCSKBGT60TR13CTOBO1 (from which you only need the radar wing)
    - If you're planning to use a different feather-wing compatible board, make sure that the "freebie" pin (the last pin on the longer header) is actually unused, as that is the CS pin for the radar.
    - Boards that have this pin free: [SparkFun Thing Plus - ESP32-S3](https://www.sparkfun.com/sparkfun-thing-plus-esp32-s3.html) and [SparkFun Thing Plus - ESP32-C6](https://www.sparkfun.com/sparkfun-thing-plus-esp32-c6.html)
- SHIELDBGT60TR13CTOBO1
- DEMOBGT60TR13CTOBO1 (which includes SHIELDBGT60TR13CTOBO1)
- CY8CKIT-062S2-AI with [CY8CKIT-062S2-AI-PASSTHROUGH](https://github.com/thedevleon/CY8CKIT-062S2-AI-PASSTHROUGH) firmware (highly experimental)

### BGT60UTR11AIP
- todo