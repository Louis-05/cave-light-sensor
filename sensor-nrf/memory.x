MEMORY
{
  /* Adafruit nRF52 UF2 bootloader layout for nRF52840:
       0x00000000..0x00027000  MBR + SoftDevice region (reserved)
       0x00027000..0x000F4000  application (820 KB)
       0x000F4000..0x00100000  bootloader + settings
   */
  FLASH : ORIGIN = 0x00027000, LENGTH = 820K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
