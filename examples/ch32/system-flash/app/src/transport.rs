//! embedded_io adapters for ch32-hal UART.

use ch32_hal as hal;
use hal::usart::{Instance, UartRx, UartTx};

// --- Async adapters ---

pub struct AsyncRx<'d, T: Instance>(pub UartRx<'d, T, hal::mode::Async>);

impl<T: Instance> embedded_io_async::ErrorType for AsyncRx<'_, T> {
    type Error = core::convert::Infallible;
}

impl<T: Instance> embedded_io_async::Read for AsyncRx<'_, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        let _ = self.0.read(&mut buf[..1]).await;
        Ok(1)
    }
}

pub struct AsyncTx<'d, T: Instance>(pub UartTx<'d, T, hal::mode::Async>);

impl<T: Instance> embedded_io_async::ErrorType for AsyncTx<'_, T> {
    type Error = core::convert::Infallible;
}

impl<T: Instance> embedded_io_async::Write for AsyncTx<'_, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let _ = self.0.write(buf).await;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// --- Blocking adapters ---

pub struct BlockingRx<'d, T: Instance>(pub UartRx<'d, T, hal::mode::Blocking>);

impl<T: Instance> embedded_io::ErrorType for BlockingRx<'_, T> {
    type Error = core::convert::Infallible;
}

impl<T: Instance> embedded_io::Read for BlockingRx<'_, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        let _ = self.0.blocking_read(&mut buf[..1]);
        Ok(1)
    }
}

pub struct BlockingTx<'d, T: Instance>(pub UartTx<'d, T, hal::mode::Blocking>);

impl<T: Instance> embedded_io::ErrorType for BlockingTx<'_, T> {
    type Error = core::convert::Infallible;
}

impl<T: Instance> embedded_io::Write for BlockingTx<'_, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let _ = self.0.blocking_write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let _ = self.0.blocking_flush();
        Ok(())
    }
}
