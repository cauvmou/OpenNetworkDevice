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

/// This is a safe low level abstraction of a Linux network interface.
pub struct Interface {
    ifr_name: [c_char; libc::IFNAMSIZ],
    ifindex: u32
}

impl Interface {
    pub fn list() -> io::Result<Vec<io::Result<Interface>>> {
        let mut ifconf: ifconf = unsafe { std::mem::zeroed() };
        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, libc::SIOCGIFCONF, &mut ifconf)?;

        ifconf.inner.ifc_req = unsafe { libc::malloc(ifconf.ifc_len as libc::size_t) as *mut ifreq};
        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, libc::SIOCGIFCONF, &mut ifconf)?;

        let interfaces = unsafe { Vec::from_raw_parts(ifconf.inner.ifc_req, ifconf.ifc_len as usize, ifconf.ifc_len as usize)};
        let interfaces: Vec<io::Result<Interface>> = interfaces.iter().map(|ifr| Interface::_open(ifr.ifr_name)).collect();

        Ok(interfaces)
    }
    pub fn open(name: &str) -> io::Result<Self> {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        let c_name = CString::new(name).expect("Failed to convert name to c_string.");
        unsafe {
            std::ptr::copy_nonoverlapping(
                c_name.as_ptr(),
                ifreq.ifr_name.as_mut_ptr() as *mut c_char,
                name.len(),
            )
        };

        Self::_open(ifreq.ifr_name)
    }

    fn _open(ifr_name: [c_char; libc::IFNAMSIZ]) -> io::Result<Self> {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        ifreq.ifr_name = ifr_name;
        ifreq.ifr_ifru.ifru_ifindex = 0;

        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, libc::SIOCGIFINDEX, &mut ifreq)?;
        let ifindex = unsafe { ifreq.ifr_ifru.ifru_ifindex } as u32;

        Ok(Self {
            ifr_name,
            ifindex
        })
    }

    pub fn up(&mut self) -> io::Result<()> {
        let mut ifreq = self._ifreq();
        unsafe { ifreq.ifr_ifru.ifru_flags |= libc::IFF_UP as c_short };
        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, libc::SIOCSIFFLAGS, &mut ifreq)?;
        Ok(())
    }

    pub fn down(&mut self) -> io::Result<()> {
        let mut ifreq = self._ifreq();
        unsafe { ifreq.ifr_ifru.ifru_flags &= !(libc::IFF_UP as c_short) };
        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, libc::SIOCSIFFLAGS, &mut ifreq)?;
        Ok(())
    }

    pub fn set_physical(&mut self, addr: [u8; 6]) -> io::Result<()> {
        todo!()
    }

    pub fn physical(&self) -> io::Result<[u8; 6]> {
        let ifreq = self._get_ifreq(libc::SIOCGIFHWADDR)?;

        let mut physical: [u8; 6] = [0; 6];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_addr.sa_data[0..6].as_ptr(),
                physical.as_mut_ptr() as *mut c_char,
                6,
            )
        };
        Ok(physical)
    }

    pub fn set_ipv4(&mut self, addr: [u8; 4]) -> io::Result<()> {
        todo!()
    }

    pub fn ipv4(&self) -> io::Result<[u8; 4]> {
        let ifreq = self._get_ifreq(libc::SIOCGIFADDR)?;
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_addr.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        Ok(ip)
    }

    pub fn set_netmask(&mut self, netmask: [u8; 4]) -> io::Result<()> {
        todo!()
    }

    pub fn netmask(&self) -> io::Result<[u8; 4]> {
        let ifreq = self._get_ifreq(libc::SIOCGIFNETMASK)?;
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_addr.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        Ok(ip)
    }

    pub fn broadcast(&self) -> io::Result<[u8; 4]> {
        let ifreq = self._get_ifreq(libc::SIOCGIFBRDADDR)?;
        let mut ip: [u8; 4] = [0; 4];
        unsafe {
            std::ptr::copy_nonoverlapping(
                ifreq.ifr_ifru.ifru_addr.sa_data[2..=5].as_ptr(),
                ip.as_mut_ptr() as *mut c_char,
                4,
            )
        };
        Ok(ip)
    }

    pub fn remove_ipv4(&mut self) -> io::Result<[u8; 4]> {
        todo!()
    }

    pub fn name(&self) -> &str {
        CStr::from_bytes_until_nul(unsafe { slice::from_raw_parts(self.ifr_name.as_ptr() as *const u8, self.ifr_name.len()) }).unwrap().to_str().unwrap()
    }

    /// Creates an ifreq with the name set to the interface name.
    ///
    /// returns: Result<ifreq, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// let ifreq: ifreq = self._ifreq();
    /// ```
    fn _ifreq(&self) -> ifreq {
        let mut ifreq: ifreq = unsafe { std::mem::zeroed() };
        ifreq.ifr_name = self.ifr_name;
        ifreq
    }

    /// Makes an ioctl request with an `ifreq` which has it's name set to the interface name.
    /// This is ideal for GET requests.
    ///
    /// # Arguments
    ///
    /// * `request`: The request to perform
    ///
    /// returns: Result<ifreq, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// let ifreq = self._get_ifreq(libc::SIOCGIFADDR)?;
    /// ```
    fn _get_ifreq(&self, request: libc::c_ulong) -> io::Result<ifreq> {
        let mut ifreq = self._ifreq();
        Self::_ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, request, &mut ifreq)?;
        Ok(ifreq)
    }


    /// A safe wrapper around sockets and ioctl requests.
    ///
    /// # Arguments
    ///
    /// * `domain`: socket domain
    /// * `ty`: socket type
    /// * `protocol`: socket protocol
    /// * `request`: The request to perform
    /// * `data`: Data for the request
    ///
    /// returns: Result<&mut T, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ifreq = self._ifreq();
    /// self._ioctl(libc::AF_INET, libc::SOCK_DGRAM, 0, request, &mut ifreq)?;
    /// ```
    fn _ioctl<T>(domain: libc::c_int, ty: libc::c_int, protocol: libc::c_int, request: libc::c_ulong, data: &mut T) -> io::Result<()> {
        let socket = unsafe { socket(domain, ty, protocol) };
        if socket == -1 {
            return Err(io::Error::last_os_error());
        }
        let result = unsafe { ioctl(socket, request, data) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        let result = unsafe { close(socket) };
        if result == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{ ipv4: {:?}, netmask: {:?}, broadcast: {:?} }}", self.name(), self.ipv4().unwrap_or_default(), self.netmask().unwrap_or_default(), self.broadcast().unwrap_or_default())
    }
}
