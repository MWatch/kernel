openocd: 
	openocd -f interface/stlink-v2.cfg -f target/stm32l4x.cfg
# openocd -f interface/stlink-v2-1.cfg -f target/stm32l4x.cfg

flash: bin
	st-flash --reset write target/thumbv7em-none-eabi/release/mabez_watch.bin 0x8000000

binfo: bin
	cargo size --release --bin mabez_watch -- -A
	ls -lha target/thumbv7em-none-eabi/release | grep mabez_watch.bin

bin: release
	cargo objcopy -- -O binary target/thumbv7em-none-eabi/release/mabez_watch target/thumbv7em-none-eabi/release/mabez_watch.bin

release:
	cargo build --release

debug:
	cargo build

clean:
	cargo clean

