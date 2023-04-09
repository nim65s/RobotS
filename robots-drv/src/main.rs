use futures::{future::FutureExt, pin_mut, select, stream::StreamExt};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use robots_drv::{recv_serial, send_serial, CmdCodec};

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut uart_port = serialport::new("/dev/ttyUSB1", 115_200).open_native_async()?;
    uart_port.set_exclusive(false)?;

    let (uart_writer, uart_reader) = CmdCodec.framed(uart_port).split();

    let t1 = recv_serial(uart_reader).fuse();
    let t2 = send_serial(uart_writer).fuse();

    // https://rust-lang.github.io/async-book/06_multiple_futures/03_select.html
    pin_mut!(t1, t2);
    select! {
        () = t1 => println!("task one completed first"),
        () = t2 => println!("task two completed first"),
    }

    Ok(())
}
