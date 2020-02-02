export tmp=`mktemp`
echo "Putting output in $tmp"
make
timeout 2 -- bash "-c 'make run'" > $tmp
#grep 'Hello riscv' $tmp
#grep 'Supervisor timer interrupt' $tmp

