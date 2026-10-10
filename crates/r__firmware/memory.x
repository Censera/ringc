/* Sectors 0 to 5 are firmware. Sectors 6 and 7 are the schedule slots (ringc-format::layout). */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 256K
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
