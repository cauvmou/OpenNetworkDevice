mod interface;
use crate::interface::Interface;

fn main() -> std::io::Result<()> {
    //Interface::list()?.iter().for_each(|interface| {
    //    match interface {
    //        Ok(interface) => {
    //            println!("{:?}", interface);
    //            println!("{} -> {}", interface.name(), interface.dot1q().unwrap());
    //        }
    //        Err(_) => {}
    //    }
    //});

    let mut interface = Interface::open("wlan0")?;
    println!("{interface:?}");
    //interface.set_ipv4([192,168,26,219])?;
    //interface.set_netmask([255,255,255,0])?;
    //interface.set_physical([40, 208, 234, 242, 179, 84])?;
    interface.remove_ipv4()?;
    println!("{interface:?}");

    let a: &'static u32 = Box::leak(Box::new(2));

    Ok(())
}
