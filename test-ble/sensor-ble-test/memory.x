MEMORY
{
  /* Seeed Xiao nRF52840 with the factory Adafruit nRF52 UF2 bootloader.
     The bootloader pre-flashes SoftDevice S140 v7.3.0.

     FLASH layout (confirmed against
     adafruit/Adafruit_nRF52_Bootloader/linker/nrf52840.ld):
       0x00000000..0x00001000  MBR (reserved by SoftDevice spec)
       0x00001000..0x00027000  SoftDevice S140 v7.3.0 (~152 KB)
       0x00027000..0x000F4000  application (820 KB)              <- us
       0x000F4000..0x000FE000  bootloader code (38 KB)
       0x000FE000..0x000FF000  MBR params page
       0x000FF000..0x00100000  bootloader settings

     RAM layout: 256 KB total (0x20000000..0x20040000).
     The SoftDevice claims the bottom N bytes for its own state. The exact
     value depends on `conn_count`, `att_mtu`, `event_length`,
     `periph_role_count` and `central_role_count` (see ble.rs::sd_config).

     For our config (1 conn, MTU 64, event_length 24, 1 periph, 0 central)
     S140 v7.3 needs roughly 6-8 KB. We reserve 16 KB (0x20004000) for
     safety margin — over-reservation only wastes RAM, under-reservation
     causes `Softdevice::enable()` to panic at boot.

     If the SD reports needing more (it logs "softdevice RAM: N bytes" via
     defmt, and panics with the required start address if defmt is wired
     up), raise the ORIGIN and reduce LENGTH by the same amount.
     If it reports needing less and you want to recover the wasted RAM,
     lower the ORIGIN.

     The upper RAM at 0x20007F7C..0x20008000 is used by the Adafruit
     bootloader for double-reset detection and (optionally) DFU bond
     sharing — these are only relevant during bootloader execution and
     are safe to overlap with the app's stack at runtime.
   */
  FLASH : ORIGIN = 0x00027000, LENGTH = 820K
  RAM   : ORIGIN = 0x20004000, LENGTH = 240K
}
