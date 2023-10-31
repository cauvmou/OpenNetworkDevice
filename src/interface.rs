use libc::{c_char, c_int, c_short, close, ifreq, ioctl, socket};
use std::ffi::CString;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct Netmask {
    octets: [u8; 4],
}

/// This is a safe low level abstraction of a Linux network interface.
pub struct Interface<'i> {
    id: u32,
    name: &'i str,
    c_name: CString,
    ifreq: ifreq,
}

impl<'i> Interface<'i> {
    pub fn open(name: &'i str) -> io::Result<Self> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        let c_name = CString::new(name).expect("Failed to convert name to c_string.");
        unsafe {
            std::ptr::copy_nonoverlapping(
                c_name.as_ptr(),
                ifreq.ifr_name.as_mut_ptr() as *mut c_char,
                name.len(),
            )
        };

        ifreq.ifr_ifru.ifru_ifindex = 0;
        let result = unsafe { ioctl(socket, libc::SIOCGIFINDEX, &ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let id = unsafe { ifreq.ifr_ifru.ifru_ifindex } as u32;
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        return Ok(Self {
            id,
            name,
            c_name,
            ifreq,
        });
    }

    pub fn up(&mut self) -> io::Result<()> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        unsafe { self.ifreq.ifr_ifru.ifru_flags |= libc::IFF_UP as c_short };
        let result = unsafe { ioctl(socket, libc::SIOCSIFFLAGS, &self.ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn down(&mut self) -> io::Result<()> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        unsafe { self.ifreq.ifr_ifru.ifru_flags &= !(libc::IFF_UP as c_short) };
        let result = unsafe { ioctl(socket, libc::SIOCSIFFLAGS, &self.ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn set_ipv4(&mut self, addr: Ipv4Addr) -> io::Result<()> {
        todo!()
    }

    pub fn set_netmask(&mut self, netmask: Ipv4Addr) -> io::Result<()> {
        todo!()
    }

    pub fn set_ipv6(&mut self, addr: Ipv6Addr) -> io::Result<()> {
        todo!()
    }

    pub fn get_ipv4(&mut self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        self.ifreq.ifr_ifru.ifru_addr = unsafe { std::mem::zeroed() };
        let result = unsafe { ioctl(socket, libc::SIOCGIFADDR, &mut self.ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.ifreq.ifr_ifru.ifru_addr.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Ipv4Addr::from(ip))
    }

    pub fn get_netmask(&mut self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        self.ifreq.ifr_ifru.ifru_netmask = unsafe { std::mem::zeroed() };
        let result = unsafe { ioctl(socket, libc::SIOCGIFNETMASK, &mut self.ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.ifreq.ifr_ifru.ifru_netmask.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Ipv4Addr::from(ip))
    }

    pub fn get_broadcast(&mut self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        self.ifreq.ifr_ifru.ifru_broadaddr = unsafe { std::mem::zeroed() };
        let result = unsafe { ioctl(socket, libc::SIOCGIFBRDADDR, &mut self.ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.ifreq.ifr_ifru.ifru_broadaddr.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Ipv4Addr::from(ip))
    }

    pub fn get_ipv6(&mut self) -> io::Result<Ipv6Addr> {
        todo!()
    }

    pub fn remove_ipv4(&mut self) -> io::Result<Ipv4Addr> {
        todo!()
    }

    pub fn remove_ipv6(&mut self) -> io::Result<Ipv6Addr> {
        todo!()
    }
}
