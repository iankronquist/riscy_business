OUTPUT_ARCH( "riscv" )
ENTRY(_start)
MEMORY
{
    ram   (wxa!ri) : ORIGIN = 0x80000000, LENGTH = 128M
}
PHDRS
{
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}

SECTIONS {

    .text : {
        __kernel_start = .;
        __text_start = .;
        /* .text.init must go first */
        *(.text.init)
        *(.text)
    } >ram AT>ram :text

    _global_pointer = .;
    .data BLOCK(4K) : ALIGN(4K) {
        __text_end = .;
        __data_start = .;
        *(.data)
    } >ram AT>ram :data

    .rodata BLOCK(4K) : ALIGN(4K) {
        __data_end = .;
        __rodata_start = .;
        *(.rodata)
    } >ram AT>ram :rodata

    .bss BLOCK(4K) : ALIGN(4K) {
        __bss_start = .;
        __rodata_end = .;
        *(.bss)
    } >ram AT>ram :bss

    . = ALIGN(4K);
    __bss_end = .;
    __kernel_end = .;
}
