INCLUDE link.x

SECTIONS
{   
  .app_section(NOLOAD) : ALIGN(4)
  {
    KEEP(*(.app_section.data));
    . = ALIGN(4);
  } > APPDATA

  .fb_section(NOLOAD) : ALIGN(4)
  {
      KEEP(*(.fb_section.fb));
      . = ALIGN(4);
  } > FRAMEBUFFER
}
