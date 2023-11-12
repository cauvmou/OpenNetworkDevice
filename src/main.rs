mod interface;
use crate::interface::Interface;

fn main() -> std::io::Result<()> {
    Interface::list()?.iter().for_each(|interface| {
        match interface {
            Ok(interface) => {
                println!("{:?}", interface);
                println!("{} -> {}", interface.name(), interface.dot1q().unwrap());
            }
            Err(_) => {}
        }
    });
    Ok(())
}
