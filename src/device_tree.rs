use crate::log;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::mem;
// Based on https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.3-rc2

const DEVICE_TREE_MAGIC: u32 = (0xd00dfeedu32).to_be();
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

pub struct DeviceTree<'a> {
    data: &'a mut [u8],
}

impl<'a> DeviceTree<'a> {
    pub fn empty() -> Self {
        Self { data: &mut [] }
    }
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

    pub fn dump(&self) {
        hexdump!(self.data);
        let iter = self.walk();
        //hexdump!(iter.bytes);
        for n in iter {
            log!("{:?}", n);
        }
    }

    pub fn find_regs(&self, name: &str) -> Option<(usize, usize)> {
        match self.find_property(name, "reg")? {
            DeviceTreeStructure::Property(slc, _) => {
                let size = match slc[0..8].try_into() {
                    Ok(arr) => u32::from_be_bytes(arr) as usize,
                    _ => {
                        return None;
                    }
                };
                let start = match slc[8..16].try_into() {
                    Ok(arr) => u32::from_be_bytes(arr) as usize,
                    _ => {
                        return None;
                    }
                };
                Some((start, size))
            }
            _ => None,
        }
    }

    pub fn find_property(&self, name: &str, prop: &str) -> Option<DeviceTreeStructure> {
        enum SearchState {
            SearchNode,
            SearchProperty,
            SearchEndProperty(usize), // depth
        }
        let mut state = SearchState::SearchNode;
        for n in self.walk() {
            match state {
                SearchState::SearchNode => match n {
                    DeviceTreeStructure::NodeBegin(node_name) => {
                        if node_name.starts_with(name)
                            && node_name.as_bytes()[name.len()] == '@' as u8
                        {
                            state = SearchState::SearchProperty;
                        }
                    }
                    _ => {
                        continue;
                    }
                },
                SearchState::SearchProperty => match n {
                    DeviceTreeStructure::Property(bytes, prop_name) => {
                        if prop_name == prop {
                            return Some(n);
                        }
                    }
                    DeviceTreeStructure::NodeBegin(_) => {
                        state = SearchState::SearchEndProperty(1);
                        continue;
                    }
                    _ => {
                        continue;
                    }
                },
                SearchState::SearchEndProperty(depth) => {
                    match n {
                        DeviceTreeStructure::NodeBegin(node_name) => {
                            state = SearchState::SearchEndProperty(depth + 1);
                            continue;
                        }
                        DeviceTreeStructure::NodeEnd => {
                            if depth == 1 {
                                // End of the node, if we haven't found the reg we aren't going to.
                                return None;
                            }
                            state = SearchState::SearchEndProperty(depth - 1);
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

    pub fn find(&self, name: &str) -> Option<usize> {
        for n in self.walk() {
            match n {
                DeviceTreeStructure::NodeBegin(node_name) => {
                    if node_name.starts_with(name) && node_name.as_bytes()[name.len()] == '@' as u8
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

    fn header(&self) -> DeviceTreeHeader {
        assert!(self.data.len() > core::mem::size_of::<DeviceTreeHeader>());
        let mut hdr: DeviceTreeHeader;
        hdr = unsafe { core::ptr::read(&self.data[0] as *const u8 as *const DeviceTreeHeader) };
        hdr
    }

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

pub struct DeviceTreeStructureIterator<'a> {
    index: usize,
    depth: isize,
    strings: &'a [u8],
    bytes: &'a [u8],
}

impl<'a> DeviceTreeStructureIterator<'a> {
    fn consume_u32(&mut self) -> Option<u32> {
        let bytes: Result<[u8; 4], _> = self.bytes[self.index..self.index + 4].try_into();
        return match bytes {
            Ok(b) => {
                self.index += 4;
                Some(u32::from_be_bytes(b))
            }
            Err(_) => None,
        };
    }
    fn consume_padding(&mut self) {
        if (self.index % 4) != 0 {
            self.index += 4 - (self.index % 4);
        }
    }
    fn consume_str(&mut self) -> Option<&'a str> {
        return match str_from_bytes(&self.bytes[self.index..]) {
            None => {
                log!("Corrupt dtb");
                None
            }
            Some(s) => {
                self.index += s.len() + 1; // null byte
                self.consume_padding();
                Some(s)
            }
        };
    }
}

impl<'a> Iterator for DeviceTreeStructureIterator<'a> {
    type Item = DeviceTreeStructure<'a>;

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
                    return Some(DeviceTreeStructure::Property(slc, name));
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

fn str_from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a str> {
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

#[derive(Debug)]
pub enum DeviceTreeStructure<'a> {
    NodeEnd,
    NodeBegin(&'a str),
    Property(&'a [u8], &'a str),
}
