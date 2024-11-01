use std::fmt::Display;

use rand::{thread_rng, RngCore};
use smoltcp::wire::{EthernetAddress, HardwareAddress};

#[derive(Debug)]
pub struct MacAddress([u8; 6]);

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let octet = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            octet[0], octet[1], octet[2], octet[3], octet[4], octet[5]
        )
    }
}

impl MacAddress {
    pub fn new() -> Self {
        // Generate random bytes
        let mut octet = [0_u8; 6];
        thread_rng().fill_bytes(&mut octet);

        // Make the address local and unicast
        octet[0] |= 0b_0000_0010;
        octet[0] &= 0b_1111_1110;

        Self(octet)
    }
}

impl Default for MacAddress {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MacAddress> for HardwareAddress {
    fn from(val: MacAddress) -> HardwareAddress {
        HardwareAddress::Ethernet(EthernetAddress(val.0))
    }
}
