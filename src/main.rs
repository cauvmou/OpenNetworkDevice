mod interface;
use crate::interface::Interface;

fn main() -> std::io::Result<()> {
    let mut interface = Interface::open("eno2")?;
    let ip = interface.get_ipv4()?;
    println!("{ip}");
    let netmask = interface.get_netmask()?;
    println!("{netmask}");
    let broadcast = interface.get_broadcast()?;
    println!("{broadcast}");
    Ok(())
}
