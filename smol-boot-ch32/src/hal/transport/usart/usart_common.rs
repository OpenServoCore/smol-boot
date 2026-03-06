use ch32_metapac::usart::Usart;

use super::Duplex;

/// Configure USART registers: baud rate, 8N1, duplex mode, then enable.
///
/// Caller must enable RCC clocks (USART + GPIO) and configure GPIO pins
/// before calling this.
pub(super) fn init(regs: &Usart, pclk: u32, baud: u32, duplex: &Duplex) {
    // Disable USART while configuring
    regs.ctlr1().write(|w| w.set_ue(false));

    // BRR = PCLK / baud (integer division; the 16x oversampling divisor
    // is already encoded in the BRR field layout)
    let brr = pclk / baud;
    regs.brr().write(|w| {
        w.set_div_mantissa((brr >> 4) as u16);
        w.set_div_fraction((brr & 0xF) as u8);
    });

    // 8N1: word length 8 bit (M=0), no parity (PCE=0), 1 stop bit (STOP=0b00)
    // (defaults are already 0, but be explicit)
    regs.ctlr2().write(|w| w.set_stop(0b00));

    // Half-duplex mode if requested
    if matches!(duplex, Duplex::Half) {
        regs.ctlr3().write(|w| w.set_hdsel(true));
    }

    // Enable USART, transmitter, and receiver
    regs.ctlr1().write(|w| {
        w.set_ue(true);
        w.set_te(true);
        w.set_re(true);
    });
}

/// Block until a byte is received, then return it.
pub(super) fn read_byte(regs: &Usart) -> u8 {
    while !regs.statr().read().rxne() {}
    regs.datar().read().dr() as u8
}

/// Block until the TX data register is empty, then write a byte.
pub(super) fn write_byte(regs: &Usart, byte: u8) {
    while !regs.statr().read().txe() {}
    regs.datar().write(|w| w.set_dr(byte as u16));
}

/// Block until the last transmission is fully complete (shift register empty).
pub(super) fn flush(regs: &Usart) {
    while !regs.statr().read().tc() {}
}
