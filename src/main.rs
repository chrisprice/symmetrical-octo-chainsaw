#![no_std]
#![no_main]

mod automation_2040w;
mod mcp23017;
mod pac_man_ball;
mod rats_nest;

use crate::{
    pac_man_ball::{Io, Outputs},
    rats_nest::RatsNest,
};
use cyw43_pio::PioSpi;
use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use edge_nal_embassy::TcpBuffers;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_rp::i2c::{self};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::{self};
use embassy_rp::{
    adc::{self},
    gpio::Output,
    peripherals::DMA_CH0,
};
use embassy_rp::{bind_interrupts, clocks::RoscRng};
use embassy_time::Duration;
use embassy_time::Timer;
use static_cell::StaticCell;

use edge_http::Method;
use edge_http::io::Error;
use edge_http::io::server::{Connection, DefaultServer, Handler};
use edge_http::ws::MAX_BASE64_KEY_RESPONSE_LEN;
use edge_nal::TcpBind;
use edge_ws::{FrameHeader, FrameType};

use embedded_io_async::{Read, Write};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn conn_led_task(mut led: Output<'static>, stack: Stack<'static>) -> ! {
    loop {
        // Slow blink until config up
        while !stack.is_config_up() {
            led.set_high();
            Timer::after(Duration::from_millis(500)).await;
            led.set_low();
            Timer::after(Duration::from_millis(500)).await;
        }

        // Fast blink until link up
        while !stack.is_link_up() {
            led.set_high();
            Timer::after(Duration::from_millis(100)).await;
            led.set_low();
            Timer::after(Duration::from_millis(100)).await;
        }

        // Solid on while link is up
        led.set_high();
        stack.wait_link_down().await;

        // If link goes down, start over (loop restarts)
        led.set_low();
    }
}

#[embassy_executor::task]
async fn http_task(stack: Stack<'static>) -> ! {
    let addr = "0.0.0.0:80";

    static BUFFERS: StaticCell<TcpBuffers<1, 512, 512>> = StaticCell::new();
    let buffers = BUFFERS.init(TcpBuffers::new());

    let tcp = edge_nal_embassy::Tcp::new(stack, buffers);
    let acceptor = tcp
        .bind(addr.parse().unwrap())
        .await
        .unwrap();

    let mut server = DefaultServer::new();
    server.run(None, acceptor, WsHandler).await.unwrap();
    defmt::unreachable!()
}


#[derive(Debug)]
enum WsHandlerError<C, W> {
    Connection(C),
    Ws(W),
}

impl<C, W> From<C> for WsHandlerError<C, W> {
    fn from(e: C) -> Self {
        Self::Connection(e)
    }
}

struct WsHandler;

impl Handler for WsHandler {
    type Error<E>
        = WsHandlerError<Error<E>, edge_ws::Error<E>>
    where
        E: core::fmt::Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl core::fmt::Display + Clone,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let headers = conn.headers()?;

        if headers.method != Method::Get {
            conn.initiate_response(405, Some("Method Not Allowed"), &[])
                .await?;
        } else if headers.path != "/" {
            conn.initiate_response(404, Some("Not Found"), &[]).await?;
        } else if !conn.is_ws_upgrade_request()? {
            conn.initiate_response(200, Some("OK"), &[("Content-Type", "text/plain")])
                .await?;

            conn.write_all(b"Initiate WS Upgrade request to switch this connection to WS")
                .await?;
        } else {
            let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
            conn.initiate_ws_upgrade_response(&mut buf).await?;

            conn.complete().await?;

            info!("Connection upgraded to WS, starting a simple WS echo server now");

            // Now we have the TCP socket in a state where it can be operated as a WS connection
            // Run a simple WS echo server here

            let mut socket = conn.unbind()?;

            let mut buf = [0_u8; 8192];

            loop {
                let mut header = FrameHeader::recv(&mut socket)
                    .await
                    .map_err(WsHandlerError::Ws)?;
                let payload = header
                    .recv_payload(&mut socket, &mut buf)
                    .await
                    .map_err(WsHandlerError::Ws)?;

                match header.frame_type {
                    FrameType::Text(_) => {
                        info!(
                            "Got {}, with payload \"{}\"",
                            header,
                            core::str::from_utf8(payload).unwrap()
                        );
                    }
                    FrameType::Binary(_) => {
                        info!("Got {}, with payload {:?}", header, payload);
                    }
                    FrameType::Close => {
                        info!("Got {}, client closed the connection cleanly", header);
                        break;
                    }
                    _ => {
                        info!("Got {}", header);
                    }
                }

                // Echo it back now

                header.mask_key = None; // Servers never mask the payload

                if matches!(header.frame_type, FrameType::Ping) {
                    header.frame_type = FrameType::Pong;
                }

                info!("Echoing back as {}", header);

                header.send(&mut socket).await.map_err(WsHandlerError::Ws)?;
                header
                    .send_payload(&mut socket, payload)
                    .await
                    .map_err(WsHandlerError::Ws)?;
            }
        }

        Ok(())
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board = automation_2040w::Automation2040W::new(p, Irqs);

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download cyw43-firmware/43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, board.pwr, board.spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(169, 254, 1, 1), 16),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), {
            let mut rng = RoscRng;
            rng.next_u64()
        });

    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(conn_led_task(board.user_led_2, stack)));

    control
        .start_ap_wpa2(
            "symmetrical-octo-chainsaw",
            option_env!("WIFI_PASSWORD").unwrap_or("default_password"),
            5,
        )
        .await;

    unwrap!(spawner.spawn(http_task(stack)));

    let mut led = board.user_led_1;
    let button = board.user_switch_a;
    let i2c = board.i2c;

    let mut ha = RatsNest::new(i2c).await.unwrap();
    let rats_nest = &mut ha;

    loop {
        let inputs = rats_nest.inputs().await.unwrap();
        info!("inputs: {:?}", inputs);

        let mut outputs = Outputs::default();
        if button.is_high() {
            led.set_high();
            outputs.checker_0_led = true;
        } else {
            led.set_low();
            outputs.checker_0_led = false;
        }
        info!("outputs: {:?}", outputs);
        rats_nest.set_outputs(outputs).await.unwrap();

        Timer::after_millis(1000).await;
    }
}
