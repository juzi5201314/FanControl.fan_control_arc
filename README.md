# Fan Control Arc

A plug-in of [FanControl](https://github.com/Rem0o/FanControl.Releases), 
which is used to provide the temperature sensor of Intel Arc Graphics.

I successfully provided gpu core temperature sensor and VRam temperature sensor on my A750.

I haven't actually learned c and c sharp (IGCL and FanControl require them), so the code for this program will appear very messy and unsound, but I'm sure I'll fix them later when I fix the problem.
This library uses rust as an intermediate layer (c# calls rust calls c), which sounds a bit redundant, but it's because i'm not familiar with c and c sharp, and don't want to use c sharp to call c ffi directly.

### Require
* The intel graphics driver must be installed. Because the IGCL(Intel Graphics Control Library) library is provided with the driver file.

### Use
1. Put the fan_control_arc_lib.dll and FanControl.fan_control_arc.dll provided by the release into {YOU_PATH}/FanControl/Plugins (same as other plugins)
2. Restart FanControl

### Known issues
* Any error: Because the code is a mess, this plugin temporarily assumes that your gpu must have a gpu temperature sensor and a gpu memory temperature sensor (otherwise it will error out).
* Set graphics card fan speed: In fact, IGCL provides related api, but it cannot be used in my A750 (4146 driver) (unsupported feature), I don't know if it is the driver or other problems. But I will test it in the next driver, if it still doesn't work I will open an issue to ask them.

### I can't use it
I am happy to solve the bug of this plugin, please open an issue and provide FanControl's logs file and your pc information (graphics card model, driver version, or other information I need).