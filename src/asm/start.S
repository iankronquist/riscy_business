#define MPP_MACHINE (0b11 << 11)
#define MPIE_YES (1 << 7)
#define MIE_YES (1 << 3)

/* Machine timer interrupt enable */
#define MIE_MTIE (1 << 7)
/* Machine external interrupt enable */
#define MIE_MEIE (1 << 11)
/* Machine software interrupt enable */
#define MIE_MSIE (1 << 3)

#define PAGE_SIZE 0x1000
.section .text.init
.global _start
_start:
    /* Disable machine interrupts */
    csrw mie, zero
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

    /* Load trap context into mscratch */
    la t0, BOOTSTRAP_CORE_TRAP_CONTEXT
    csrw mscratch, t0

    la t1, _trap_stack_top
    sd t1, (32*8)(t0)

    /* Load trap vector into mtvec */
    la t0, _trap
    csrw mtvec, t0

    /* MPIE is whether interrupts were enabled prior to the last trap in M
    mode. */
    /* MIE is machine interrupts enabled */
    /* MPP is the previous privilege level */
    li t0, MPP_MACHINE | MPIE_YES | MIE_YES
    li t0, MPP_MACHINE
    csrw mstatus, t0

    /* Enable machine interrupts */
    li t0, (MIE_MEIE | MIE_MSIE)
    /*li t0, (MIE_MTIE | MIE_MEIE | MIE_MSIE)*/
    csrw mie, t0

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
.skip 4 * PAGE_SIZE
_stack_top:
/* FIXME we should probably allocate this from a real allocator */
_trap_stack_bottom:
.skip 4 * PAGE_SIZE
_trap_stack_top:
