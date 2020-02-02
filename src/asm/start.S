#define MPP_MACHINE (0b11 << 11)
#define MPP_SUPERVISOR (0b01 << 11)
#define SPP_SUPERVISOR (1 << 8)
#define MPIE_YES (1 << 7)
#define SPIE_YES (1 << 5)
#define MIE_YES (1 << 3)
#define SIE_YES (1 << 1)

/* Machine external interrupt enable */
#define MIE_MEIE (1 << 11)
/* Machine timer interrupt enable */
#define MIE_MTIE (1 << 7)
/* Machine software interrupt enable */
#define MIE_MSIE (1 << 3)

/* Machine external interrupt enable */
#define SIE_SEIE (1 << 9)
/* Supervisor timer interrupt enable */
#define SIE_STIE (1 << 5)
/* Machine software interrupt enable */
#define SIE_SSIE (1 << 1)

#define MACHINE_MODE_START 1

#define PAGE_SIZE 0x1000
.section .text.init
.global _start
_start:
    /* mhartid is in a0. Device tree pointer is in a1. Do not disturb them */
#if MACHINE_MODE_START
    /* Disable machine interrupts */
    csrw mie, zero
    /* Delegate all exceptions and interrupts to supervisor mode */
    li t0, ~0
    csrw mideleg, t0
    csrw medeleg, t0
    /* We will return to supervisor mode */
    li t0, MPP_SUPERVISOR
    csrw mstatus, t0
    la t0, .lower_to_smode
    csrw mepc, t0
    mret
    /* we should never get here, but if we do, hang */
    j hang
#endif

.lower_to_smode:
    /* mhartid is in a0. Park non-init cores */
    bnez    a0, hang

    /* SATP should be zero (like CR3 in x86), but let's make sure */
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
    csrw sscratch, t0

    /* Set trap stack in the trap context */
    la t1, _trap_stack_top
    sd t1, (32*8)(t0)

    /* Load trap vector into mtvec */
    la t0, _trap
    csrw stvec, t0

    /* SPIE is whether interrupts were enabled prior to the last trap in S
    mode. */
    /* SIE is machine interrupts enabled */
    /* SPP is the previous privilege level */
    li t0, SPP_SUPERVISOR | SPIE_YES | SIE_YES
    csrw sstatus, t0

    /* Enable supervisor interrupts */
    li t0, (SIE_STIE | SIE_SEIE | SIE_SSIE)
    csrw sie, t0

    /* Set machine exception PC to rmain so we will return to it */
    /* This is the equivalent of the elr */
    la t1, rmain
    csrw sepc, t1
    /* Return into supervisor mode code */
    sret

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
