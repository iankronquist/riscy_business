/*
use crate::constants::{PAGE_SIZE, LAST_GIANT_PAGE, MemoryAccess, MemoryCacheability};

type VAddr = usize;
type PAddr = usize;

pub struct AddressSpace {
    satp: PAddr,
}

struct PageTableEntry(usize);

impl PageTableEntry {
    fn is_leaf(&self) -> bool {
        (self.0 & (PTE_R | PTE_W | PTE_X)) != 0
    }

    fn is_valid(&self) -> bool {
        (self.0 & PTE_V) == PTE_V
    }

    fn access_to_arch(access: MemoryAccess) -> usize {
        match access {
            SupervisorReadable => PTE_R,
            SupervisorWritable => PTE_R | PTE_W,   // implies Readable.
            SupervisorExecutable => PTE_R | PTE_X, // implies Readable but not Writable.
            UserReadable => PTE_U | PTE_R,         // can be read by supervisor if you use copy_from_user.
            UserWritable => PTE_U | PTE_R | PTE_W,         // can be written by supervisor if you use copy_to_user.
            UserExecutable => PTE_U | PTE_R | PTE_X,       // implies not *Writable.
        }
    }
    fn from_paddr(phys: PAddr, access: MemoryAccess) -> Self {
        assert!((phys & 0xfff) != 0);
        Self(phys | access_to_arch(access) | PTE_V)
    }

    fn to_paddr(&self) -> &PAddr {
        &(self.0 & PTE_PPN_MASK)
    }

    // We deliberately DO NOT impl drop here, because there maybe shared memory
    // mappings, e.g. the kernel itself, which we do not want to unmap.
    fn release(&mut self) {


        self.0 = 0;
    }
}


struct PageTableLevel {
    entries: [PageTableEntry; 512],
}

const SATP_MODE_NONE: usize = 0;
const SATP_MODE_SV39: usize = 8 << 60;
const SATP_MODE_SV48: usize = 9 << 60;

// Section  4.3.1, fig 4.15
const PTE_PPN_MASK: usize = 0x3ffffffffff000;
const PTE_RSW_COW: usize = 1 << 8;
const PTE_DIRTY: usize = 1 << 7;
const PTE_ACCESSED: usize = 1 << 6;
const PTE_GLOBAL: usize = 1 << 5;
const PTE_USER: usize = 1 << 4;
const PTE_X: usize = 1 << 3;
const PTE_W: usize = 1 << 2;
const PTE_R: usize = 1 << 1;
const PTE_V: usize = 1 << 0;

const PTE_INDEX_MASK: usize = (1 << 9) - 1;

const SATP_MODE_MASK: usize = 0xf << 60;
const SATP_ASID_MASK: usize = 0xff << 44;
const SATP_PPN_MASK: usize = (1 << 44) - 1;
const PAGE_OFFSET: usize = 12;

fn phys_to_ppn(phys: PAddr) -> usize {
    phys / PAGE_SIZE
}

fn ppn_to_phys(ppn: usize) -> PAddr{
    ppn * PAGE_SIZE
}


const MEMORY_WINDOW_START: VAddr = LAST_GIANT_PAGE;

// If you have more than 512 GB of physical memory you might have issues :P
fn phys_to_virt(phys: PAddr) -> VAddr {
    MEMORY_WINDOW_START + phys
}

impl AddressSpace {
    fn verify_satp(&self) -> bool {
        self.satp & SATP_MODE_MASK == SATP_MODE_SV48
    }
    pub fn switch(&self) {
        assert!(self.verify_satp());
        unsafe { asm!("csrw satp, $0" :: "r"(self.satp) ::"volatile"); }
    }
    fn top_pt(&self) -> *mut PageTableLevel {
        let va = phys_to_virt(ppn_to_phys(self.satp & SATP_PPN_MASK));
        assert!(va >= MEMORY_WINDOW_START);
        va as *mut PageTableLevel
    }
    fn pte_to_page_table_level(pte: PageTableEntry) -> *mut PageTableLevel {
        assert!((pte & PTE_V) == PTE_V);
        phys_to_virt(pte & PTE_PPN_MASK) as *mut PageTableLevel
    }

    fn identity_map_kernel() { }


    pub fn map(&mut self, virt: VAddr, phys: PAddr, access: MemoryAccess) -> Result<(), ()> {
        let mut pt = self.top_pt();
        for i in 3..0 {
            let idx = ((virt >> PAGE_OFFSET) >> (9 * i)) & PTE_INDEX_MASK;
            let mut pte = unsafe { &mut (*pt).entries.[idx] };
            // Is leaf, already present.
            if pte.is_leaf() {
                return Err(());
            }
            // Reached the bottom.
            if i == 0 {
                pte = PageTableEntry::from_paddr(phys, access);
                return Ok(());

            } else if !pte.is_valid() {
                // If absent, allocate.
                phys::alloc_range(1);
            }
        }
        let mut pte = unsafe { (*pt).entries.[idx] };
        // Trying to map a present page...
        if pte != 0 {
            return Err(());
        }
        pte = unsafe { (*pt).entries.[idx] };
    }

    }
}
*/
