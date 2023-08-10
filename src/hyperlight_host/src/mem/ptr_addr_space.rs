use super::{layout::SandboxMemoryLayout, shared_mem::SharedMemory};
use anyhow::Result;

/// A representation of a specific address space
pub(crate) trait AddressSpace: std::cmp::Eq {
    /// The base address for this address space
    fn base(&self) -> u64;
}

/// The address space for the guest executable
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GuestAddressSpace(u64);
impl GuestAddressSpace {
    /// Create a new instance of a `GuestAddressSpace`
    pub(crate) fn new() -> Result<Self> {
        let base_addr = u64::try_from(SandboxMemoryLayout::BASE_ADDRESS)?;
        Ok(Self(base_addr))
    }
}
impl AddressSpace for GuestAddressSpace {
    fn base(&self) -> u64 {
        self.0
    }
}

/// The address space for the host executable
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct HostAddressSpace(u64);
impl HostAddressSpace {
    /// Create a new instance of a `HostAddressSpace`, using the given
    /// `SharedMemory` as the base address.
    pub(crate) fn new(shared_mem: &SharedMemory) -> Result<Self> {
        let base = u64::try_from(shared_mem.base_addr())?;
        Ok(Self(base))
    }
}
impl AddressSpace for HostAddressSpace {
    fn base(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::{layout::SandboxMemoryLayout, shared_mem::SharedMemory};

    use super::{AddressSpace, GuestAddressSpace, HostAddressSpace};

    #[test]
    fn host_addr_space_base() {
        let gm = SharedMemory::new(10).unwrap();
        let space = HostAddressSpace::new(&gm).unwrap();
        assert_eq!(gm.base_addr() as u64, space.base());
    }

    #[test]
    fn guest_addr_space_base() {
        let space = GuestAddressSpace::new().unwrap();
        assert_eq!(SandboxMemoryLayout::BASE_ADDRESS as u64, space.base());
    }
}
