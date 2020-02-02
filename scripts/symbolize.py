import re
import sys
import subprocess

STACK_TRACE_BEGIN="= Stack trace ="
STACK_TRACE_END="==============="
TOOLCHAIN='riscv64-elf'
ADDR2LINE=TOOLCHAIN+'-addr2line'
#if len(sys.argv) > 0:
#    for f in sys.argv[1:]:
#        for line in open(f, 'r').readlines():
#else:


def filter_syms(stream):
    stack_trace_found = False
    for line in stream.readlines():
        if STACK_TRACE_BEGIN in line:
            stack_trace_found = True
        elif STACK_TRACE_END in line:
            stack_trace_found = False
        elif stack_trace_found:
            m = re.match(r'([xa-f0-9A-F]+)...([xa-f0-9A-F]+)', line)
            if m:
                fp, pc = m.groups()
                result = subprocess.run([ADDR2LINE, '-f', '-C', '-e', 'riscy_business.elf', pc], capture_output=True)
                print('{} | {} {}'.format(fp,pc,result.stdout.strip().decode('utf8')))#, end='')
                continue

        print(line, end='')


filter_syms(sys.stdin)
