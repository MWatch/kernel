target remote :3333

# print demangled symbols
set print asm-demangle on

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break UserHardFault
break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

monitor tpiu config internal itm.fifo uart off 16000000
monitor itm port 0 on

load

# start the process but immediately halt the processor
stepi