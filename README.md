Ultra Low power light sensor that logs event when luminosity exceed a threshold.

Hardware :
- STM32L011K4Tx
- OPT3001 light sensor
- ~~micro SD card reader~~ removed because it overflowed the 2KB RAM when trying to read/write to the SDcard + took to much flash memory (+10KB) because of the library