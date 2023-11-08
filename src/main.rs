mod interface;
use crate::interface::Interface;

fn main() -> std::io::Result<()> {
    let mut interfaces = Interface::list()?.iter().for_each(|interface| {
        match interface {
            Ok(interface) => {
                println!("{:?}", interface)
            }
            Err(_) => {}
        }
    });
    Ok(())
}
