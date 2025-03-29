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
        Self::test_preset()
    }
}

impl core::fmt::Display for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let chirp_repetition_time_hz = 1.0 / self.chirp_repetition_time_s;
        let frame_repetition_time_hz = 1.0 / self.frame_repetition_time_s;
        let frame_shape: [u64; 3] = [
            self.rx_antennas.into(),
            self.num_chirps_per_frame.into(),
            self.num_samples_per_chirp.into(),
        ];
        let frame_buffer_size_u12: u64 = frame_shape.iter().product(); // product is only available for u64
        let frame_buffer_size_u8 = (frame_buffer_size_u12 * 12) / 8;

        write!(
            f,
            "Config {{\n\
            \t rx_antennas: {},\n\
            \t tx_antennas: {},\n\
            \t tx_power_level: {},\n\
            \t if_gain_db: {},\n\
            \t lower_frequency_hz: {},\n\
            \t upper_frequency_hz: {},\n\
            \t num_chirps_per_frame: {},\n\
            \t num_samples_per_chirp: {},\n\
            \t chirp_repetition_time_s: {:.2e},\n\
            \t frame_repetition_time_s: {:.2e},\n\
            \t sample_rate_hz: {},\n\
            \t chirp_repetition_time_hz: {:.2e},\n\
            \t frame_repetition_time_hz: {:.2e},\n\
            \t frame_shape: {:?},\n\
            \t frame_buffer_size_u12: {},\n\
            \t frame_buffer_size_u8: {}\n\
            \t registers: {:?},\n\
            }}",
            self.rx_antennas,
            self.tx_antennas,
            self.tx_power_level,
            self.if_gain_db,
            self.lower_frequency_hz,
            self.upper_frequency_hz,
            self.num_chirps_per_frame,
            self.num_samples_per_chirp,
            self.chirp_repetition_time_s,
            self.frame_repetition_time_s,
            self.sample_rate_hz,
            chirp_repetition_time_hz,
            frame_repetition_time_hz,
            frame_shape,
            frame_buffer_size_u12,
            frame_buffer_size_u8,
            self.registers,
        )?;

        Ok(())
    }
}

impl Config {
    pub fn test_preset() -> Self {
        Config::new(
            1,
            1,
            31,
            60,
            61020099000,
            61479903000,
            1,
            128,
            6.21125e-05,
            0.0998265,
            2352941,
            [
                0x11e8270, 0x3088210, 0x9e967fd, 0xb0805b4, 0xd1027ff, 0xf010700, 0x11000000,
                0x13000000, 0x15000000, 0x17000be0, 0x19000000, 0x1b000000, 0x1d000000, 0x1f000b60,
                0x21103c51, 0x231ff41f, 0x25006f7b, 0x2d000490, 0x3b000480, 0x49000480, 0x57000480,
                0x5911be0e, 0x5b678c0a, 0x5d000000, 0x5f787e1e, 0x61f5208a, 0x630000a4, 0x65000252,
                0x67000080, 0x69000000, 0x6b000000, 0x6d000000, 0x6f093910, 0x7f000100, 0x8f000100,
                0x9f000100, 0xad000000, 0xb7000000,
            ],
        )
    }

    pub fn high_framerate_preset() -> Self {
        Config::new(
            3,
            1,
            31,
            60,
            61020099000,
            61479903000,
            16,
            128,
            6.99625e-05,
            0.0050039,
            2352941,
            [
                0x11e8270, 0x3088210, 0x9e967fd, 0xb0805b4, 0xd1027ff, 0xf010700, 0x11000000,
                0x13000000, 0x15000000, 0x17000be0, 0x19000000, 0x1b000000, 0x1d000000, 0x1f000b60,
                0x21130c51, 0x234ff41f, 0x25006f7b, 0x2d000490, 0x3b000480, 0x49000480, 0x57000480,
                0x5911be0e, 0x5b3ef40a, 0x5d00f000, 0x5f787e1e, 0x61f5208a, 0x630000a4, 0x65000252,
                0x67000080, 0x69000000, 0x6b000000, 0x6d000000, 0x6f093910, 0x7f000100, 0x8f000100,
                0x9f000100, 0xad000000, 0xb7000000,
            ],
        )
    }
}
