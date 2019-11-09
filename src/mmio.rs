pub unsafe fn write(address: usize, offset: usize, value: u8) {
    let reg = address as *mut u8;
    reg.add(offset).write_volatile(value);
}

pub unsafe fn read(address: usize, offset: usize, value: u8) -> u8 {
    let reg = address as *mut u8;
    reg.add(offset).read_volatile()
}
