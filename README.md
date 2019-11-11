# Riscy Business
Screwing around with risc-v. All based on the osblog tutorial series here, with considerable alterations:
http://osblog.stephenmarz.com

We use a device tree parser instead of hard-coding a bunch of addresses. The API for the parser could use a little work but accomplishes all of the basic functionality.
I also have some nice goodies like a safer abstraction over MMIO.
The tutorial is mired in machine mode (the moral equivalent of EL3) which isn't so hot, so we drop down into supervisor mode ASAP. Machine mode means there's no paging, which is a total security nightmare.
Because they don't have paging in their kernel, their physical memory manager has a questionable design.
