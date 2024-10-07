use esp_idf_svc::{
    eth::{BlockingEth, EspEth, EthDriver},
    eventloop::EspSystemEventLoop,
    hal::{gpio::*, prelude::*, spi::{config::DriverConfig, Dma, SpiDriver}},
    log::EspLogger,
};
use esp_idf_svc::hal::gpio;

use std::net::Ipv4Addr;
use std::str::FromStr;

use esp_idf_svc::ipv4::{
    ClientConfiguration as IpClientConfiguration, ClientSettings as IpClientSettings,
    Configuration as IpConfiguration, Mask, Subnet,
};
use esp_idf_svc::netif::{EspNetif, NetifConfiguration};

const DEVICE_IP: &str = "192.168.1.242";
const GATEWAY_IP: &str = "192.168.1.1";
const GATEWAY_NETMASK: Option<&str> = Some("24");

use log::info;


fn main() -> anyhow::Result<()> {
    use std::sync::Mutex;

    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;
    let sys_loop = EspSystemEventLoop::take()?;

    let netmask = GATEWAY_NETMASK.unwrap_or("24");
    let netmask = u8::from_str(netmask)?;
    let gateway_addr = Ipv4Addr::from_str(GATEWAY_IP)?;
    let static_ip = Ipv4Addr::from_str(DEVICE_IP)?;

    let gpio12 = pins.gpio12;
    let mut phy = PinDriver::output(gpio12)?;
    let _ = phy.set_high()?;
    // let mphy = Mutex::new(phy);
    // let lphy =mphy.lock();

    // Make sure to configure ethernet in sdkconfig and adjust the parameters below for your hardware
    let eth_driver = EthDriver::new_rmii(
        peripherals.mac,
        pins.gpio25,
        pins.gpio26,
        pins.gpio27,
        pins.gpio23,
        pins.gpio22,
        pins.gpio21,
        pins.gpio19,
        pins.gpio18,
        esp_idf_svc::eth::RmiiClockConfig::<gpio::Gpio0, gpio::Gpio16, gpio::Gpio17>::OutputInvertedGpio17(
            pins.gpio17,
        ),
        Some(pins.gpio0),
        // Replace with IP101 if you have that variant, or with some of the others in the `RmiiEthChipset`` enum
        esp_idf_svc::eth::RmiiEthChipset::LAN87XX,
        Some(0),
        sys_loop.clone(),
    )?;
    let eth = EspEth::wrap_all(
        eth_driver,
        EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: IpConfiguration::Client(IpClientConfiguration::Fixed(
                IpClientSettings {
                    ip: static_ip,
                    subnet: Subnet {
                        gateway: gateway_addr,
                        mask: Mask(netmask),
                    },
                    // Can also be set to Ipv4Addrs if you need DNS
                    dns: None,
                    secondary_dns: None,
                },
            )),
            ..NetifConfiguration::wifi_default_client()
        })?,
    )?;

    info!("Eth created");

    let mut eth = BlockingEth::wrap(eth, sys_loop.clone())?;

    info!("Starting eth...");

    eth.start()?;

    eth.wait_netif_up()?;

    let ip_info = eth.eth().netif().get_ip_info()?;

    info!("Eth DHCP info: {:?}", ip_info);

    Ok(())
}