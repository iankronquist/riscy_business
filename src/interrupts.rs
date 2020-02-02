pub fn disable() {
    unsafe { asm!("csrw sie, zero; wfi;"::::"volatile"); }
}

pub fn wait() {
    unsafe { asm!("csrw sie, zero; wfi;"::::"volatile"); }
}

pub fn info() {
    let mut sie: usize;
    let mut sip: usize;
    let mut stvec: usize;
    let mut stval: usize;
    let mut scause: usize;
    let mut sstatus: usize;
    let mut sepc: usize;

    unsafe {
        asm!("csrr $0, sie" : "=r"(sie));
        asm!("csrr $0, sip" : "=r"(sip));
        asm!("csrr $0, stvec" : "=r"(stvec));
        asm!("csrr $0, stval" : "=r"(stval));
        asm!("csrr $0, scause" : "=r"(scause));
        asm!("csrr $0, sstatus" : "=r"(sstatus));
        asm!("csrr $0, sepc" : "=r"(sepc));
    }

    log!("sie: {:x?}\nsip: {:x?}\nstvec: {:x?}\nstval: {:x?}\nscause: {:x?}\nsstatus: {:x?}\nsepc: {:x?}", sie, sip, stvec, stval, scause, sstatus, sepc);
}
