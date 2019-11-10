#define MPP_MACHINE (0b11 << 11)
#define MPIE_YES (1 << 7)
#define MIE_YES (1 << 3)

/* Machine timer interrupt enable */
#define MIE_MTIE (1 << 7)
/* Machine external interrupt enable */
#define MIE_MEIE (1 << 11)
/* Machine software interrupt enable */
#define MIE_MSIE (1 << 11)

#define PAGE_SIZE 0x1000
.section .text.init
.global _start
_start:
    /* Device tree pointer is in a1. Something else is in a0. Do not disturb
    them */
    /* Any hardware threads (hart) that are not bootstrapping */
    /* need to wait for an IPI */
    csrr    t0, mhartid
    bnez    t0, hang
    /* SATP should be zero, but let's make sure */
    csrw    satp, zero
    /* Zero BSS */
    la t0, __bss_start
    la t1, __bss_end
    bgeu t0, t1, 2f
1:
    sd zero, (t0)
    addi t0, t0, 8
    bltu t0, t1, 1b
2:
    /* Set up stack */
    la sp, _stack_top
    /* Set global pointer to the beginning of data/rodata */
    la gp, _global_pointer

    /* MPIE is whether interrupts were enabled prior to the last trap in M
    mode. */
    /* MIE is machine interrupts enabled */
    /* MPP is the previous privilege level */
    /* li t0, MPP_MACHINE | MPIE_YES | MIE_YES */
    li t0, MPP_MACHINE
    csrw mstatus, t0

    /* csrw mie, t3 */

    /* Set machine exception PC to rmain so we will return to it */
    /* This is the equivalent of the elr */
    la t1, rmain
    csrw mepc, t1
    /* Return into machine mode code */
    mret


hang:
    wfi
    j hang


.section .data
_stack_bottom:
.skip 16 * PAGE_SIZE
_stack_top:
