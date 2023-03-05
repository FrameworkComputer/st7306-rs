/// ST7735 instructions.
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    NOP = 0x00,
    /// Software Reset
    SWRESET = 0x01,
    /// Read Display ID
    RDDID = 0x04,
    /// Read Display Status
    RDDST = 0x09,
    /// Sleep In
    SLPIN = 0x10,
    /// Sleep Out
    SLPOUT = 0x11,
    /// Partial On
    PTLON = 0x12,
    /// Partial Off
    PTLOFF = 0x13,
    /// Display Inversion Off
    INVOFF = 0x20,
    /// Display Inversion On
    INVON = 0x21,
    // Display Off
    DISPOFF = 0x28,
    // Display On
    DISPON = 0x29,
    /// Column Address Set
    CASET = 0x2A,
    /// Row Address Set
    RASET = 0x2B,
    // Memory Write
    RAMWR = 0x2C,
    /// Tearing Effect Line Offf
    TEOFF = 0x34,
    /// Tearing Effect Line On
    TEON = 0x35,
    /// Memory Data Access Control
    MADCTL = 0x36,
    // Vertial Scroll Start Address of RAM
    VSCSAD = 0x37,
    /// High Power Mode ON
    HPM = 0x38,
    /// Low Power Mode On
    LPM = 0x39,
    // Data Format Select
    DTFORM = 0x3A,
    // Write Memory Continue
    WRMEMC = 0x3C,
    // Set Tear Scanline
    TESCAN = 0x44,

    /// Gate Timing Control
    GTCON = 0x62,
    /// Gate Line Setting
    GATESET = 0xB0,
    /// First Gate Setting
    FSTCOM = 0xB1,
    /// Frame Rate Control
    FRCTRL = 0xB2,
    /// Update Period Gate EQ Control in HPM
    GTUPEQH = 0xB3,
    /// Update Period Gate EQ Control in LPM
    GTUPEQL = 0xB4,
    /// Source EQ Enable
    SOUEQ = 0xB7,
    /// Panel Setting
    PNLSET = 0xB8,
    /// Gamma Mode Setting
    GAMAMS = 0xB9,
    /// Enable Clear RAM
    CLRAM = 0xBB,
    /// Gate Voltage Control
    GCTRL = 0xC0,
    /// Source High Positive Voltage Control
    VSHPCTRL = 0xC1,
    /// Source Low Positive Voltage Control
    VSLPCTRL = 0xC2,
    /// Source High Negative Voltage Control
    VSHNCTRL = 0xC4,
    /// Source Low Negative Voltage Control
    VSLNCTRL = 0xC5,
    /// Ultra low power (Undocumented)
    LOWPOWER = 0xC7,
    /// Source Gamma Voltage Control
    VSIKCTRL = 0xC8,
    /// Source Voltage Select
    VSHLSEL = 0xC9,
    /// ID1 Setting
    ID1SET = 0xCA,
    /// ID2 Setting
    ID2SET = 0xCB,
    /// ID3 Setting
    ID3SET = 0xCC,
    /// Enable Auto Power Down
    AUTOPWRCTRL = 0xD0,
    /// Booster Enable
    BSTEN = 0xD1,
    /// NVM Load Control
    NVMLOADCTRL = 0xD6,
    /// OSC Setting
    OSCSET = 0xD8,
    /// NVM Data Read
    NVMRD = 0xE9,
    /// EXTB Control
    EXTBCTRL = 0xEC,
    /// NVM WR/RD Control
    NVMCTRL1 = 0xF8,
    /// NVM Program Setting
    NVMCTRL2 = 0xFA,
    /// NVM REad Enable
    NVMRDEN = 0xFB,
    /// NVM Program Enable
    NVMPROM = 0xFC,

    /// Read ID1
    RDID1 = 0xDA,
    /// Read ID2
    RDID2 = 0xDB,
    /// Read ID3
    RDID3 = 0xDC,
}
