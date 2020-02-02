#define REG_SIZE 8
#define CPU_MAX 8
#define SAVE_GP(i, base) sd x##i, (i*REG_SIZE)(t6)

.section .text
.global _trap
.align 4
_trap:
    /* I presume interrupts are off? */
    /* swap t6, sscratch */
    /* sscratch holds trap state context */
    csrrw t6, sscratch, t6

    sd x0, (0*REG_SIZE)(t6)
    sd x1, (1*REG_SIZE)(t6)
    sd x2, (2*REG_SIZE)(t6)
    sd x3, (3*REG_SIZE)(t6)
    sd x4, (4*REG_SIZE)(t6)
    sd x5, (5*REG_SIZE)(t6)
    sd x6, (6*REG_SIZE)(t6)
    sd x7, (7*REG_SIZE)(t6)
    sd x8, (8*REG_SIZE)(t6)
    sd x9, (9*REG_SIZE)(t6)
    sd x10, (10*REG_SIZE)(t6)
    sd x11, (11*REG_SIZE)(t6)
    sd x12, (12*REG_SIZE)(t6)
    sd x13, (13*REG_SIZE)(t6)
    sd x14, (14*REG_SIZE)(t6)
    sd x15, (15*REG_SIZE)(t6)
    sd x16, (16*REG_SIZE)(t6)
    sd x17, (17*REG_SIZE)(t6)
    sd x18, (18*REG_SIZE)(t6)
    sd x19, (19*REG_SIZE)(t6)
    sd x20, (20*REG_SIZE)(t6)
    sd x21, (21*REG_SIZE)(t6)
    sd x22, (22*REG_SIZE)(t6)
    sd x23, (23*REG_SIZE)(t6)
    sd x24, (24*REG_SIZE)(t6)
    sd x25, (25*REG_SIZE)(t6)
    sd x26, (26*REG_SIZE)(t6)
    sd x27, (27*REG_SIZE)(t6)
    sd x28, (28*REG_SIZE)(t6)
    sd x29, (29*REG_SIZE)(t6)
    sd x30, (30*REG_SIZE)(t6)

    /* sscratch has the contents of the previous context's t6, swap it back
     * into t6 and save it */
    mv t5, t6
    csrr t6, sscratch
    sd x31, (31*REG_SIZE)(t5)
    /* Restore trap context into sscratch */
    csrw sscratch, t5

    csrr a0, sepc
    csrr a1, stval
    csrr a2, scause
    /* csrr a3, shartid */
    /* FIXME: mhartid went away when we became smode */
    xor a3, a3, a3
    csrr a4, sstatus
    mv a5, t5 /* trap context */
    /* We are now on a trusted stack */
    ld sp, (32*REG_SIZE)(t5)

    call rtrap

    /* machine exception pc gets return value of rtrap */
    csrw sepc, a0

    /* t6 gets trap context */
    csrr t6, sscratch

    ld x31, (31*REG_SIZE)(t6)
    ld x30, (30*REG_SIZE)(t6)
    ld x29, (29*REG_SIZE)(t6)
    ld x28, (28*REG_SIZE)(t6)
    ld x27, (27*REG_SIZE)(t6)
    ld x26, (26*REG_SIZE)(t6)
    ld x25, (25*REG_SIZE)(t6)
    ld x24, (24*REG_SIZE)(t6)
    ld x23, (23*REG_SIZE)(t6)
    ld x22, (22*REG_SIZE)(t6)
    ld x21, (21*REG_SIZE)(t6)
    ld x20, (20*REG_SIZE)(t6)
    ld x19, (19*REG_SIZE)(t6)
    ld x18, (18*REG_SIZE)(t6)
    ld x17, (17*REG_SIZE)(t6)
    ld x16, (16*REG_SIZE)(t6)
    ld x15, (15*REG_SIZE)(t6)
    ld x14, (14*REG_SIZE)(t6)
    ld x13, (13*REG_SIZE)(t6)
    ld x12, (12*REG_SIZE)(t6)
    ld x11, (11*REG_SIZE)(t6)
    ld x10, (10*REG_SIZE)(t6)
    ld x9, (9*REG_SIZE)(t6)
    ld x8, (8*REG_SIZE)(t6)
    ld x7, (7*REG_SIZE)(t6)
    ld x6, (6*REG_SIZE)(t6)
    ld x5, (5*REG_SIZE)(t6)
    ld x4, (4*REG_SIZE)(t6)
    ld x3, (3*REG_SIZE)(t6)
    ld x2, (2*REG_SIZE)(t6)
    ld x1, (1*REG_SIZE)(t6)
    ld x0, (0*REG_SIZE)(t6)
    /* Leap! */
    sret
