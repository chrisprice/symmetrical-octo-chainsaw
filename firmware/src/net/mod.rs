use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals::PIO0;
use embassy_rp::{gpio::Output, peripherals::DMA_CH0};
use embassy_time::Duration;
use embassy_time::Timer;
use static_cell::StaticCell;

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

        info!("Network config: {:?}", stack.config_v4());

        // Fast blink until link up
        while !stack.is_link_up() {
            led.set_high();
            Timer::after(Duration::from_millis(100)).await;
            led.set_low();
            Timer::after(Duration::from_millis(100)).await;
        }

        info!("Link is up");

        // Solid on while link is up
        led.set_high();

        stack.wait_link_down().await;

        info!("Link is down");

        led.set_low();
    }
}

pub async fn init(
    spawner: Spawner,
    ssid: &str,
    passphrase: &str,
    channel: u8,
    conn_led: Output<'static>,
    pwr: Output<'static>,
    spi: PioSpi<'static, PIO0, 0, DMA_CH0>,
) -> Stack<'static> {
    let fw = include_bytes!("../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download cyw43-firmware/43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(192, 168, 0, 1), 24),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    static RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), {
            let mut rng = RoscRng;
            rng.next_u64()
        });

    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(conn_led_task(conn_led, stack)));

    control.start_ap_wpa2(ssid, passphrase, channel).await;

    stack
}
