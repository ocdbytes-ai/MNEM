// Fundamental
pub const SET_CONTRAST: u8 = 0x81;
pub const DISPLAY_RAM: u8 = 0xA4;
pub const DISPLAY_ALL_ON: u8 = 0xA5;
pub const NORMAL_DISPLAY: u8 = 0xA6;
pub const INVERT_DISPLAY: u8 = 0xA7;
pub const DISPLAY_OFF: u8 = 0xAE;
pub const DISPLAY_ON: u8 = 0xAF;
pub const PIXEL_ON: u8 = 0xFF;
pub const PIXEL_OFF: u8 = 0x00;

// Scrolling
pub const SCROLL_RIGHT: u8 = 0x26;
pub const SCROLL_LEFT: u8 = 0x27;
pub const SCROLL_VERT_RIGHT: u8 = 0x29;
pub const SCROLL_VERT_LEFT: u8 = 0x2A;
pub const SCROLL_DEACTIVATE: u8 = 0x2E;
pub const SCROLL_ACTIVATE: u8 = 0x2F;
pub const SET_VERT_SCROLL_AREA: u8 = 0xA3;

// Addressing
pub const SET_MEM_ADDR_MODE: u8 = 0x20;
pub const SET_COL_ADDR: u8 = 0x21;
pub const SET_PAGE_ADDR: u8 = 0x22;

pub mod addr_mode {
    pub const HORIZONTAL: u8 = 0x00;
    pub const VERTICAL: u8 = 0x01;
    pub const PAGE: u8 = 0x02;
}

// Hardware configuration
pub const SET_START_LINE: u8 = 0x40; // OR with 0x00–0x3F for offset
pub const SEG_REMAP_NORMAL: u8 = 0xA0;
pub const SEG_REMAP_FLIP: u8 = 0xA1;
pub const SET_MULTIPLEX: u8 = 0xA8;
pub const COM_SCAN_NORMAL: u8 = 0xC0;
pub const COM_SCAN_FLIP: u8 = 0xC8;
pub const SET_DISPLAY_OFFSET: u8 = 0xD3;
pub const SET_COM_PINS: u8 = 0xDA;

pub mod com_pins {
    pub const SEQUENTIAL: u8 = 0x02;
    pub const ALT_128X64: u8 = 0x12; // use this for 128x64 displays
    pub const ALT_128X32: u8 = 0x02; // use this for 128x32 displays
}

// Timing and driving
pub const SET_DISPLAY_CLOCK: u8 = 0xD5;
pub const SET_PRECHARGE: u8 = 0xD9;
pub const SET_VCOMH: u8 = 0xDB;
pub const NOP: u8 = 0xE3;

pub mod vcomh {
    pub const V065: u8 = 0x00;
    pub const V077: u8 = 0x20; // default
    pub const V083: u8 = 0x30;
    pub const V100: u8 = 0x40;
}

// Charge pump
pub const CHARGE_PUMP: u8 = 0x8D;

pub mod pump {
    pub const ENABLE: u8 = 0x14;
    pub const DISABLE: u8 = 0x10;
}

// I2C prefix bytes
pub const PREFIX_CMD: u8 = 0x00;
pub const PREFIX_DATA: u8 = 0x40;

// Display geometry
pub const WIDTH: u8 = 128;
pub const HEIGHT: u8 = 64;
pub const PAGES: u8 = 8; // HEIGHT / 8
pub const FRAMEBUFFER_SIZE: usize = 1024; // WIDTH * PAGES

// Ready-made init sequence for 128x64 with charge pump
pub const INIT_SEQUENCE: &[u8] = &[
    DISPLAY_OFF,
    SET_DISPLAY_CLOCK,
    0x80,
    SET_MULTIPLEX,
    0x3F,
    SET_DISPLAY_OFFSET,
    0x00,
    SET_START_LINE,
    CHARGE_PUMP,
    pump::ENABLE,
    SET_MEM_ADDR_MODE,
    addr_mode::HORIZONTAL,
    SEG_REMAP_FLIP,
    COM_SCAN_FLIP,
    SET_COM_PINS,
    com_pins::ALT_128X64,
    SET_CONTRAST,
    0xCF,
    SET_PRECHARGE,
    0xF1,
    SET_VCOMH,
    vcomh::V077,
    DISPLAY_RAM,
    NORMAL_DISPLAY,
    DISPLAY_ON,
];

// I2C slave address
pub const I2C_ADDR: u8 = 0x3c;
