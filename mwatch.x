INCLUDE link.x

SECTIONS
{   
  .app_section :
  {
    KEEP(*(.app_section.data));
  } > APPDATA
}
