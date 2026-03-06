use core::convert::Infallible;

use embedded_io::ErrorType;

pub(crate) struct Ch32UartTransport;

impl Ch32UartTransport {
    pub fn new() -> Self {
        Ch32UartTransport
    }
}

impl ErrorType for Ch32UartTransport {
    type Error = Infallible;
}

impl embedded_io::Read for Ch32UartTransport {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl embedded_io::Write for Ch32UartTransport {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
