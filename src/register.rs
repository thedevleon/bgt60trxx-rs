
use bitfield_struct::bitfield;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(u8)]
pub enum Register {
    MAIN = 0x00,
    ADC0 = 0x01,
    CHIP_ID = 0x02,
    STAT1 = 0x03,
    PACR1 = 0x04,
    PACR2 = 0x05,
    SFCTL = 0x06,
    SADC_CTRL = 0x07,
    CSI_0 = 0x08,
    CSI_1 = 0x09,
    CSI_2 = 0x0A,
    CSCI = 0x0B,
    CSDS_0 = 0x0C,
    CSDS_1 = 0x0D,
    CSDS_2 = 0x0E,
    CSCDS = 0x0F,
    CSU1_0 = 0x10,
    CSU1_1 = 0x11,
    CSU1_2 = 0x12,
    CSD1_0 = 0x13,
    CSD1_1 = 0x14,
    CSD1_2 = 0x15,
    CSC1 = 0x16,
    CSU2_0 = 0x17,
    CSU2_1 = 0x18,
    CSU2_2 = 0x19,
    CSD2_0 = 0x1A,
    CSD2_1 = 0x1B,
    CSD2_2 = 0x1C,
    CSC2 = 0x1D,
    CSU3_0 = 0x1E,
    CSU3_1 = 0x1F,
    CSU3_2 = 0x20,
    CSD3_0 = 0x21,
    CSD3_1 = 0x22,
    CSD3_2 = 0x23,
    CSC3 = 0x24,
    CSU4_0 = 0x25,
    CSU4_1 = 0x26,
    CSU4_2 = 0x27,
    CSD4_0 = 0x28,
    CSD4_1 = 0x29,
    CSD4_2 = 0x2A,
    CSC4 = 0x2B,
    CCR0 = 0x2C,
    CCR1 = 0x2D,
    CCR2 = 0x2E,
    CCR3 = 0x2F,
    PLL1_0 = 0x30,
    PLL1_1 = 0x31,
    PLL1_2 = 0x32,
    PLL1_3 = 0x33,
    PLL1_4 = 0x34,
    PLL1_5 = 0x35,
    PLL1_6 = 0x36,
    PLL1_7 = 0x37,
    PLL2_0 = 0x38,
    PLL2_1 = 0x39,
    PLL2_2 = 0x3A,
    PLL2_3 = 0x3B,
    PLL2_4 = 0x3C,
    PLL2_5 = 0x3D,
    PLL2_6 = 0x3E,
    PLL2_7 = 0x3F,
    PLL3_0 = 0x40,
    PLL3_1 = 0x41,
    PLL3_2 = 0x42,
    PLL3_3 = 0x43,
    PLL3_4 = 0x44,
    PLL3_5 = 0x45,
    PLL3_6 = 0x46,
    PLL3_7 = 0x47,
    PLL4_0 = 0x48,
    PLL4_1 = 0x49,
    PLL4_2 = 0x4A,
    PLL4_3 = 0x4B,
    PLL4_4 = 0x4C,
    PLL4_5 = 0x4D,
    PLL4_6 = 0x4E,
    PLL4_7 = 0x4F,
    RFT0 = 0x55,
    RFT1 = 0x56,
    PLL_DFT0 = 0x59,
    STAT0 = 0x5D,
    SADC_RESULT = 0x5E,
    FSTAT = 0x5F,
    FIFO = 0x60
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct MAIN {
    pub frame_start: bool,
    pub sw_reset: bool,
    pubfsm_reset: bool,
    #[bits(8)]
    pub tr_wkup: usize,
    #[bits(4)]
    pub tw_wkup_mul: usize,
    pub cw_mode: bool,
    #[bits(2)] 
    pub sadc_clkdiv: usize,
    #[bits(2)]
    pub bg_clk_div: usize,
    #[bits(2)]
    pub load_strength: usize,
    pub ldo_mode: bool,
    #[bits(9)] // padding
    __: usize,
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct CHIP_ID {
    #[bits(8)]
    pub digital_id: usize,
    #[bits(16)]
    pub rf_id: usize,
    #[bits(8)]
    __: usize,
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct STAT1 {
    #[bits(12)]
    pub shape_grp_cnt: usize,
    #[bits(12)]
    pub frame_cnt: usize,
    #[bits(8)]
    __: usize,
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct SFCTL {
    #[bits(13)]
    pub fifo_cref: usize,
    pub fifo_lp_mode: bool,
    #[bits(2)]
    __: usize,
    pub miso_hs_rd: bool,    
    pub lfsr_en: bool,
    pub prefix_en: bool,
    #[bits(13)]
    __: usize,
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct STAT0 {
    pub sadc_rdy: bool,
    pub madc_rdy: bool,
    pub madc_bgup: bool,
    pub ldo_rdy: bool,
    pub __: bool,
    #[bits(3)]
    pub pm: usize,
    #[bits(3)]
    pub ch_idx: usize,
    #[bits(3)]
    pub sd_idx: usize,
    #[bits(18)]
    pub __: usize,    
}

#[bitfield(u32)]
#[allow(non_camel_case_types)]
struct FSTAT {
    #[bits(14)]
    fill_status: usize,
    #[bits(3)]
    __: usize,
    pub clk_num_err: bool,
    pub spi_burst_err: bool,
    pub fuf_err: bool,
    pub empty: bool,
    pub cref: bool,
    pub full: bool,
    pub fof_err: bool,
    #[bits(8)]
    __: usize,
}