/// The configuration of the BGT60TR13C radar sensor, mostly used for reference only.
/// The actual configuration is done via the generated register list.
/// 
/// The fields of the configuration match the fields of the JSON required for the bgt60-configurator-cli.
pub struct Config {
    pub rx_antennas: u8,
    pub tx_antennas: u8,
    pub tx_power_level: u8,
    pub if_gain_db: u8,
    pub lower_frequency_hz: u64,
    pub upper_frequency_hz: u64,
    pub num_chirps_per_frame: u8,
    pub num_samples_per_chirp: u16,
    pub chirp_repetition_time_s: f64,
    pub frame_repetition_time_s: f64,
    pub sample_rate_hz: u32,
    pub registers: [u32; 38],
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rx_antennas: u8,
        tx_antennas: u8,
        tx_power_level: u8,
        if_gain_db: u8,
        lower_frequency_hz: u64,
        upper_frequency_hz: u64,
        num_chirps_per_frame: u8,
        num_samples_per_chirp: u16,
        chirp_repetition_time_s: f64,
        frame_repetition_time_s: f64,
        sample_rate_hz: u32,
        registers: [u32; 38],
    ) -> Self {
        Config {
            rx_antennas,
            tx_antennas,
            tx_power_level,
            if_gain_db,
            lower_frequency_hz,
            upper_frequency_hz,
            num_chirps_per_frame,
            num_samples_per_chirp,
            chirp_repetition_time_s,
            frame_repetition_time_s,
            sample_rate_hz,
            registers,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(
            3,
            1,
            31,
            60,
            61020098000,
            61479902000,
            16,
            128,
            7e-05,
            5e-3,
            2330000,
            [
                0x11e8270, 0x3088210, 0x9e967fd, 0xb0805b4, 0xdf02fff, 0xf010700, 0x11000000,
                0x13000000, 0x15000000, 0x17000be0, 0x19000000, 0x1b000000, 0x1d000000, 0x1f000b60,
                0x21130c51, 0x234ff41f, 0x25006f7b, 0x2d000490, 0x3b000480, 0x49000480, 0x57000480,
                0x5911be0e, 0x5b3ef40a, 0x5d00f000, 0x5f787e1e, 0x61f5208c, 0x630000a4, 0x65000252,
                0x67000080, 0x69000000, 0x6b000000, 0x6d000000, 0x6f092910, 0x7f000100, 0x8f000100,
                0x9f000100, 0xad000000, 0xb7000000,
            ],
        )
    }
}

/*
Example configuration JSON file:
{
    "device_config": {
        "fmcw_single_shape": {
            "rx_antennas": [3],
            "tx_antennas": [1],
            "tx_power_level": 31,
            "if_gain_dB": 60,
            "lower_frequency_Hz": 61020098000,
            "upper_frequency_Hz": 61479902000,
            "num_chirps_per_frame": 16,
            "num_samples_per_chirp": 128,
            "chirp_repetition_time_s": 7e-05,
            "frame_repetition_time_s": 5e-3,
            "sample_rate_Hz": 2330000
        }
    }
}

And the generated header file:
/* XENSIV BGT60TRXX register configurator, SDK versionv3.3.0+207.a6ebda979 */

#ifndef XENSIV_BGT60TRXX_CONF_MICRO_H
#define XENSIV_BGT60TRXX_CONF_MICRO_H

#define XENSIV_BGT60TRXX_CONF_DEVICE (XENSIV_DEVICE_BGT60TR13C)
#define XENSIV_BGT60TRXX_CONF_START_FREQ_HZ (61020100000)
#define XENSIV_BGT60TRXX_CONF_END_FREQ_HZ (61479904000)
#define XENSIV_BGT60TRXX_CONF_NUM_SAMPLES_PER_CHIRP (128)
#define XENSIV_BGT60TRXX_CONF_NUM_CHIRPS_PER_FRAME (16)
#define XENSIV_BGT60TRXX_CONF_NUM_RX_ANTENNAS (1)
#define XENSIV_BGT60TRXX_CONF_NUM_TX_ANTENNAS (1)
#define XENSIV_BGT60TRXX_CONF_SAMPLE_RATE (2352941)
#define XENSIV_BGT60TRXX_CONF_CHIRP_REPETITION_TIME_S (6.945e-05)
#define XENSIV_BGT60TRXX_CONF_HIGH_FRAME_REPETITION_TIME_S (0.0049961)
#define XENSIV_BGT60TRXX_CONF_NUM_REGS_MICRO (38)


static uint32_t register_list_micro_only[] = {
    0x11e8270UL,
    0x3088210UL,
    0x9e967fdUL,
    0xb0805b4UL,
    0xdf02fffUL,
    0xf010700UL,
    0x11000000UL,
    0x13000000UL,
    0x15000000UL,
    0x17000be0UL,
    0x19000000UL,
    0x1b000000UL,
    0x1d000000UL,
    0x1f000b60UL,
    0x21130c51UL,
    0x234ff41fUL,
    0x25006f7bUL,
    0x2d000490UL,
    0x3b000480UL,
    0x49000480UL,
    0x57000480UL,
    0x5911be0eUL,
    0x5b3ef40aUL,
    0x5d00f000UL,
    0x5f787e1eUL,
    0x61f5208cUL,
    0x630000a4UL,
    0x65000252UL,
    0x67000080UL,
    0x69000000UL,
    0x6b000000UL,
    0x6d000000UL,
    0x6f092910UL,
    0x7f000100UL,
    0x8f000100UL,
    0x9f000100UL,
    0xad000000UL,
    0xb7000000UL
};

#endif /* XENSIV_BGT60TRXX_CONF_MICRO_H */
*/
