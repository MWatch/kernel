# SWO SETUP 
- Debugger pinout https://www.waveshare.com/img/devkit/general/connector-layout-20pin_h220.jpg

```
        Board ------------ Debugger 
Pins
        VCC(5v)             VDD (3.3v)
        GND(Gnd)            GND
        SWO(PB3)            TDO(Pin 13 on header) // IMPORT FOR TRACE PRINTF!
        TCLK(SWCLK)         TCLK(pin 9 on header)
        TIO(SWIO)           SWIO(pin 7 on header)
        TVCC(V+ on board)   TVCC(1 or 2) // required for debugging
```

