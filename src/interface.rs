use libc::{c_char, c_int, c_short, close, ifreq, ioctl, socket};
use std::ffi::{CStr, CString};
use std::{io, slice};
use std::fmt::{Debug, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::slice_from_raw_parts;

#[repr(C)]
union __c_anonymous_ifconf {
    ifc_buf: *mut c_char,
    ifc_req: *mut ifreq,
}

#[repr(C)]
struct ifconf {
    ifc_len: c_int,
    inner: __c_anonymous_ifconf
}

pub struct Netmask {
    octets: [u8; 4],
}

/// This is a safe low level abstraction of a Linux network interface.
pub struct Interface {
    ifr_name: [c_char; libc::IFNAMSIZ],
    ifindex: u32
}

impl Interface {
    pub fn list() -> io::Result<Vec<io::Result<Interface>>> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ifconf: ifconf = unsafe { std::mem::zeroed() };

        let result = unsafe { ioctl(socket, libc::SIOCGIFCONF, &ifconf) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        ifconf.inner.ifc_req = unsafe { libc::malloc(ifconf.ifc_len as libc::size_t) as *mut ifreq};
        let result = unsafe { ioctl(socket, libc::SIOCGIFCONF, &ifconf) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        let interfaces = unsafe { Vec::from_raw_parts(ifconf.inner.ifc_req, ifconf.ifc_len as usize, ifconf.ifc_len as usize)};

        let interfaces: Vec<io::Result<Interface>> = interfaces.iter().map(|ifr| Interface::_open(ifr.ifr_name)).collect();

        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(interfaces)
    }
    pub fn open(name: &str) -> io::Result<Self> {
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
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        Self::_open(ifreq.ifr_name)
    }

    pub fn _open(ifr_name: [c_char; libc::IFNAMSIZ]) -> io::Result<Self> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        ifreq.ifr_name = ifr_name;
        ifreq.ifr_ifru.ifru_ifindex = 0;
        let result = unsafe { ioctl(socket, libc::SIOCGIFINDEX, &ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let ifindex = unsafe { ifreq.ifr_ifru.ifru_ifindex } as u32;
        let result = unsafe { libc::close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }

        return Ok(Self {
            ifr_name,
            ifindex
        });
    }

    pub fn up(&mut self) -> io::Result<()> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        let mut ifreq = self._ifreq();
        unsafe { ifreq.ifr_ifru.ifru_flags |= libc::IFF_UP as c_short };
        let result = unsafe { ioctl(socket, libc::SIOCSIFFLAGS, &ifreq) };
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
        let mut ifreq = self._ifreq();
        unsafe { ifreq.ifr_ifru.ifru_flags &= !(libc::IFF_UP as c_short) };
        let result = unsafe { ioctl(socket, libc::SIOCSIFFLAGS, &ifreq) };
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

    pub fn get_ipv4(&self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut ifreq = self._ifreq();
        let result = unsafe { ioctl(socket, libc::SIOCGIFADDR, &mut ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_addr.sa_data[2..=5].as_ptr(),
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

    pub fn get_netmask(&self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut ifreq = self._ifreq();
        let result = unsafe { ioctl(socket, libc::SIOCGIFNETMASK, &mut ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_netmask.sa_data[2..=5].as_ptr(),
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

    pub fn get_broadcast(&self) -> io::Result<Ipv4Addr> {
        let socket = unsafe { socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut ifreq = self._ifreq();
        let result = unsafe { ioctl(socket, libc::SIOCGIFBRDADDR, &mut ifreq) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_broadaddr.sa_data[2..=5].as_ptr(),
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

    pub fn remove_ipv4(&mut self) -> io::Result<Ipv4Addr> {
        todo!()
    }

    pub fn get_name(&self) -> &str {
        CStr::from_bytes_until_nul(unsafe { slice::from_raw_parts(self.ifr_name.as_ptr() as *const u8, self.ifr_name.len()) }).unwrap().to_str().unwrap()
    }

    fn _ifreq(&self) -> ifreq {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        ifreq.ifr_name = self.ifr_name;
        ifreq
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{ ip: {:?}, netmask: {:?}, broadcast: {:?} }}", self.get_name(), self.get_ipv4(), self.get_netmask(), self.get_broadcast())
    }
}
