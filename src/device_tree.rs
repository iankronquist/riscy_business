use crate::log;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::mem;

const DEVICE_TREE_MAGIC: u32 = (0xd00d_feedu32).to_be();
const DEVICE_TREE_COMPAT_VERSION: u32 = (16u32).to_be();
const DEVICE_TREE_CURRENT_VERSION: u32 = (17u32).to_be();

// Tokens are converted to native endian after consumption.
const FDT_BEGIN_NODE: u32 = 1;
const FDT_END_NODE: u32 = 2;
const FDT_PROP_NODE: u32 = 3;
const FDT_NOP_NODE: u32 = 4;
const FDT_END: u32 = 9;

#[repr(C)]
pub struct FdtPropData {
    len: u32,
    nameoff: u32,
}
// All values are big endian

#[derive(Copy, Clone)]
#[repr(C)]
struct DeviceTreeHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

#[repr(C)]
pub struct DeviceTreeMemoryReservationEntry {
    address: u64,
    size: u64,
}

/// Device tree blob parser.
/// Based on the v0.3-rc2 specification found here:
/// https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.3-rc2
pub struct DeviceTree<'dtb> {
    data: &'dtb mut [u8],
}

impl<'dtb> DeviceTree<'dtb> {
    /// Creates a new device tree blob with no data.
    pub fn empty() -> Self {
        Self { data: &mut [] }
    }
    /// Creates a device tree blob at the given address.
    /// The blob's size and metadata is created by dereferencing the given
    /// address and treating it as a header, so this method is unsafe.
    /// Returns a DeviceTree object if the magic, version, and size information
    /// is all valid. Otherwise returns None.
    pub unsafe fn from_address(addr: usize) -> Option<Self> {
        let dtb = addr as *mut DeviceTreeHeader;
        if (*dtb).magic != DEVICE_TREE_MAGIC {
            log!(
                "Device Tree Blob: bad magic: {:x} {:x}",
                (*dtb).magic,
                DEVICE_TREE_MAGIC
            );
            return None;
        }
        if (*dtb).last_comp_version != DEVICE_TREE_COMPAT_VERSION
            && (*dtb).version != DEVICE_TREE_CURRENT_VERSION
        {
            log!(
                "Device Tree Blob: bad version {:x} {:x} {:x}",
                (*dtb).version,
                (*dtb).last_comp_version,
                DEVICE_TREE_COMPAT_VERSION
            );
            return None;
        }
        let size = u32::from_be((*dtb).totalsize);
        let off_dt_struct = u32::from_be((*dtb).off_dt_struct);
        let size_dt_struct = u32::from_be((*dtb).size_dt_struct);
        let off_dt_strings = u32::from_be((*dtb).off_dt_strings);
        let size_dt_strings = u32::from_be((*dtb).size_dt_strings);
        if (size as usize) < mem::size_of::<DeviceTreeHeader>()
            || size < off_dt_struct + size_dt_struct
            || size < off_dt_strings + size_dt_strings
        {
            log!("Device Tree Blob: bad size");
            return None;
        }
        Some(Self {
            data: core::slice::from_raw_parts_mut(
                dtb as *mut u8,
                u32::from_be((*dtb).totalsize) as usize,
            ),
        })
    }

    /// Dump the device tree to the debug log.
    pub fn dump(&self) {
        hexdump!(self.data);
        let iter = self.walk();
        //hexdump!(iter.bytes);
        for n in iter {
            log!("{:?}", n);
        }
    }

    pub fn find_regs(&self, name: &str) -> Option<(usize, usize)> {
        let prop = self.find_property(name, "reg")?;
        let start = prop.read_usize(0)?;
        let size = prop.read_usize(8)?;
        Some((start, size))
    }

    /// Find the first property matching the first object with the given
    /// property and name.
    pub fn find_property(&self, name: &str, prop: &str) -> Option<DeviceTreeNodeProperty> {
        enum SearchState {
            Node,
            Propery,
            EndProperty(usize), // depth
        }
        let mut state = SearchState::Node;
        for n in self.walk() {
            match state {
                SearchState::Node => match n {
                    DeviceTreeStructure::NodeBegin(node_name) => {
                        if node_name.starts_with(name) && node_name.as_bytes()[name.len()] == b'@' {
                            state = SearchState::Propery;
                        }
                    }
                    _ => {
                        continue;
                    }
                },
                SearchState::Propery => match n {
                    DeviceTreeStructure::Property(node_prop) => {
                        if node_prop.name == prop {
                            return Some(node_prop);
                        }
                    }
                    DeviceTreeStructure::NodeBegin(_) => {
                        state = SearchState::EndProperty(1);
                        continue;
                    }
                    _ => {
                        continue;
                    }
                },
                SearchState::EndProperty(depth) => {
                    match n {
                        DeviceTreeStructure::NodeBegin(node_name) => {
                            state = SearchState::EndProperty(depth + 1);
                            continue;
                        }
                        DeviceTreeStructure::NodeEnd => {
                            if depth == 1 {
                                // End of the node, if we haven't found the reg we aren't going to.
                                return None;
                            }
                            state = SearchState::EndProperty(depth - 1);
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }
        }
        None
    }

    /// Given the name 'X', finds the item `X@HEX_ADDRESS` and returns the hex
    /// address as an Option<usize>. If the name is not found or does not have
    /// that form, return None.
    pub fn find(&self, name: &str) -> Option<usize> {
        for n in self.walk() {
            match n {
                DeviceTreeStructure::NodeBegin(node_name) => {
                    if node_name.starts_with(name)
                        && node_name.as_bytes().len() > name.len()
                        && node_name.as_bytes()[name.len()] == b'@'
                    {
                        if let Ok(addr) = usize::from_str_radix(&node_name[name.len() + 1..], 16) {
                            return Some(addr);
                        }
                    }
                }
                _ => continue,
            }
        }
        None
    }

    /// Returns a copy of the device tree header.
    fn header(&self) -> DeviceTreeHeader {
        assert!(self.data.len() > core::mem::size_of::<DeviceTreeHeader>());
        let mut hdr: DeviceTreeHeader;
        // FIXME: Find a better way to do this...
        hdr = unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            core::ptr::read(&self.data[0] as *const u8 as *const DeviceTreeHeader)
        };
        hdr
    }

    /// Returns an iterator which will walk the DeviceTree in depth first
    /// order.
    pub fn walk(&self) -> DeviceTreeStructureIterator {
        assert!(core::mem::size_of::<usize>() >= core::mem::size_of::<u32>());
        let hdr = self.header();
        let start = u32::from_be(hdr.off_dt_struct) as usize;
        let end = (u32::from_be(hdr.off_dt_struct) + u32::from_be(hdr.size_dt_struct)) as usize;
        let dtb_structure = &self.data[start..end];

        let strings_start = u32::from_be(hdr.off_dt_strings) as usize;
        let strings_end =
            (u32::from_be(hdr.off_dt_strings) + u32::from_be(hdr.size_dt_strings)) as usize;
        let dtb_strings = &self.data[strings_start..strings_end];
        DeviceTreeStructureIterator {
            index: 0,
            depth: 0,
            bytes: dtb_structure,
            strings: dtb_strings,
        }
    }
}

/// Iterates over the contents of the DeviceTree and performs minimal
/// validation.
pub struct DeviceTreeStructureIterator<'dtb> {
    index: usize,
    depth: isize,
    strings: &'dtb [u8],
    bytes: &'dtb [u8],
}

#[derive(Debug)]
pub struct DeviceTreeNodeProperty<'dtb> {
    pub bytes: &'dtb [u8],
    pub name: &'dtb str,
}

impl<'dtb> DeviceTreeNodeProperty<'dtb> {
    fn read_u64(&self, offset: usize) -> Option<u64> {
        const LEN: usize = mem::size_of::<u64>();
        if offset + LEN > self.bytes.len() {
            return None;
        }
        let bytes: Result<[u8; LEN], _> = self.bytes[offset..offset + LEN].try_into();
        match bytes {
            Ok(b) => {
                Some(u64::from_be_bytes(b))
            }
            Err(_) => None,
        }
    }

    fn read_usize(&self, offset: usize) -> Option<usize> {
        if offset + LEN > self.bytes.len() {
            return None;
        }
        const LEN: usize = mem::size_of::<usize>();
        let bytes: Result<[u8; LEN], _> = self.bytes[offset..offset + LEN].try_into();
        match bytes {
            Ok(b) => {
                Some(usize::from_be_bytes(b))
            }
            Err(_) => None,
        }
    }

    fn new(bytes: &'dtb [u8], name: &'dtb str) -> Self {
        Self { bytes, name }
    }

}

/// Elements of the device tree structure and their values.
#[derive(Debug)]
pub enum DeviceTreeStructure<'dtb> {
    NodeBegin(&'dtb str),
    NodeEnd,
    Property(DeviceTreeNodeProperty<'dtb>)
}


impl<'dtb> DeviceTreeStructureIterator<'dtb> {
    fn consume_u32(&mut self) -> Option<u32> {
        let bytes: Result<[u8; 4], _> = self.bytes[self.index..self.index + 4].try_into();
        match bytes {
            Ok(b) => {
                self.index += 4;
                Some(u32::from_be_bytes(b))
            }
            Err(_) => None,
        }
    }
    fn consume_padding(&mut self) {
        if (self.index % 4) != 0 {
            self.index += 4 - (self.index % 4);
        }
    }
    fn consume_str(&mut self) -> Option<&'dtb str> {
        match str_from_bytes(&self.bytes[self.index..]) {
            None => {
                log!("Corrupt dtb");
                None
            }
            Some(s) => {
                self.index += s.len() + 1; // null byte
                self.consume_padding();
                Some(s)
            }
        }
    }
}

impl<'dtb> Iterator for DeviceTreeStructureIterator<'dtb> {
    type Item = DeviceTreeStructure<'dtb>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.bytes.len() {
            let token = self.consume_u32()?;
            match token {
                // Nop nodes are ignored, bump the index and parse the next node.
                FDT_NOP_NODE => continue,
                FDT_BEGIN_NODE => {
                    self.depth += 1;
                    let s = self.consume_str()?;
                    return Some(DeviceTreeStructure::NodeBegin(s));
                }
                FDT_END_NODE => {
                    self.depth -= 1;
                    if self.depth < 0 {
                        log!("Unbalanced dtb nodes");
                        return None;
                    }
                    return Some(DeviceTreeStructure::NodeEnd);
                }
                FDT_PROP_NODE => {
                    let len = self.consume_u32()? as usize;
                    let nameoff = self.consume_u32()? as usize;
                    let start = self.index;
                    let end = self.index + len;
                    // Consume byte blob.
                    self.index += len;
                    self.consume_padding();
                    let slc = &self.bytes[start..end];
                    // Do not consume the name since it lives in the strings section.
                    let name = str_from_bytes(&self.strings[nameoff..])?;
                    return Some(DeviceTreeStructure::Property(DeviceTreeNodeProperty::new(slc, name)));
                }
                FDT_END => {
                    if self.depth != 0 {
                        log!("Unbalanced dtb nodes");
                    }
                    // We're done.
                    return None;
                }
                _ => {
                    log!(
                        "Corrupt dtb, bad token 0x{:x} offset 0x{:x}",
                        token,
                        self.index
                    );
                    return None;
                }
            }
        }
        None
    }
}

/// Given a byte slice, return C-style null terminated utf8 str reference.
/// This is useful for parsing out property and node names.
fn str_from_bytes(bytes: &'_ [u8]) -> Option<&'_ str> {
    let mut len = 0;
    while len < bytes.len() {
        if bytes[len] == 0 {
            return match core::str::from_utf8(&bytes[..len]) {
                Ok(s) => Some(s),
                Err(_) => None,
            };
        }
        len += 1;
    }
    // The array is empty or we fell off the end without finding a null byte.
    None
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::fs::File;
    use std::vec::Vec;
    #[test]
    fn parse_device_tree() {
        let data = std::fs::read("./tests/riscv-virt.dtb").unwrap();
        let dtb = unsafe { DeviceTree::from_address(&data[0] as *const u8 as usize) }.unwrap();
        let uart_spot = dtb.find("uart").unwrap();
        assert_eq!(uart_spot, 0x10000000);
        let pci_spot = dtb.find("pci").unwrap();
        assert_eq!(pci_spot, 0x30000000);
        let interrupt_controller_spot = dtb.find("interrupt-controller").unwrap();
        assert_eq!(interrupt_controller_spot, 0xc000000);

        let prop = dtb.find_property("uart", "reg").unwrap();
        assert_eq!(prop.read_u64(0), Some(0x10000000));
        assert_eq!(prop.read_u64(8), Some(0x100));
        assert_eq!(prop.read_u64(9), None);
        assert_eq!(prop.read_u64(16), None);
        let uart_regs = dtb.find_regs("uart").unwrap();
        assert_eq!(uart_regs, (0x10000000, 0x100));
    }
}
