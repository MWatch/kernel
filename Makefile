openocd: 
	openocd -f interface/stlink-v2.cfg -f target/stm32l4x.cfg
# openocd -f interface/stlink-v2-1.cfg -f target/stm32l4x.cfg

flash: bin
	st-flash --reset write target/thumbv7em-none-eabi/release/mwatch_kernel.bin 0x8000000

binfo: bin
	cargo size --release --bin mwatch_kernel -- -A
	ls -lha target/thumbv7em-none-eabi/release | grep mwatch_kernel.bin

bin: release
	cargo objcopy -- -O binary target/thumbv7em-none-eabi/release/mwatch_kernel target/thumbv7em-none-eabi/release/mwatch_kernel.bin

release:
	cargo build --release

debug:
	cargo build

clean:
	cargo clean

