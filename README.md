# w25n01gv-rs
This project implements a driver for Winbond W25N01GVxxIG/IT flash chips. Because there are no embedded-hal traits for QSPI, for now I've directly added a dependency to the stm32l4xx-hal library that I'll personally be using to interface with the flash chips. I may write a temporary QSPI trait in the future, and ideally I'll shift the library to use any QSPI traits from embedded-hal when (if) they come out. Until then feel free to adapt the library to your own needs. 

Some basic examples can be found in the examples folder. `write_read` writes a couple values to the first page of the first block and reads it back via semihosting. `validate` continually writes and reads back pages sequentially in the first block and alerts when bytes read back incorrectly. This is useful for checking QSPI bus speeds, wire length, interference, etc.

# Gotcha's
One thing to note that I don't believe is clearly explained in the data sheet: writing must be sequential. These flash chips are broken into blocks, and each block is broken down into pages. Within a block, pages must be written sequentially from lowest address to highest address. If you attempt to write a page out of order, it will *silently* corrupt the data in that page. Random reads are fine, but random writes are not.