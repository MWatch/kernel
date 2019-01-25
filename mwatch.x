INCLUDE link.x

SECTIONS
{   
  .app_section :
  {
    KEEP(*(.app_section.data));
  } > APPDATA

  .fb_section :
  {
      KEEP(*(.fb_section.fb));
  } > FRAMEBUFFER
}
