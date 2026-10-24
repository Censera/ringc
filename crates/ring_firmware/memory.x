/* Pages 0 to 47 are firmware. Slots A and B follow (ring_format::layout). */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 48K
    RAM   : ORIGIN = 0x20000000, LENGTH = 20K
}
