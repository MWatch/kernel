


openocd: 
	openocd -f interface/stlink-v2-1.cfg -f target/stm32l4x.cfg

flash: release bin
	st-flash write target/thumbv7em-none-eabi/release/mabez_watch.bin 0x8000000

bin:
	cargo objcopy -- -O binary target/thumbv7em-none-eabi/release/mabez_watch target/thumbv7em-none-eabi/release/mabez_watch.bin

binfo:
	cargo size --release --bin mabez_watch -- -A
	ls -lha target/thumbv7em-none-eabi/release | grep mabez_watch.bin

release:
	cargo build --release

debug:
	cargo build

