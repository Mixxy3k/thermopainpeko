# For debbuging
# openocd -s C:\share\scripts -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
# For flashing
# cargo flash --release --chip STM32F303VCTx

target extended-remote :3333
monitor arm semihosting enable
load
