# Greaheisl User Manual

This manual describes the behaviour of the real Arduino Uno R4 Wifi when the software provided in this repository is loaded. It also describes the behaviour of the emulator on the PC. On the emulator, you can see what the staus of the relays would be in reality in the line above the emulated LED display.

## idle display

When you connect your device for the first time (or run the emulator) or when the device is idle, the LED display changes the displayed content every once in a while. Right now there are only two idle display modes implemented:

* a clock (hours and minutes shown)
* a flower (static image)

In the beginning, the clock is hard to read. The numbers are just two pixels wide, but that was the only way to fit the clock on the 12 x 8 LED matrix. 

If you do not want to change the idle display mode immediately, you can use the `Previous` or `Next` button.

## setting the clock

When the clock is shown, hold down the `Enter` button until the first two digits start blinking. Use the `Previous` or `Next` button to set the hour. Press `Enter` to be able to set the minutes. Note that you can hold the `Previous` or `Next` button down long to be able to increase/decrease the minutes in steps of 10. Press `Enter` once more. The clock blinks three times quickly to 
confirm your setting.

During the procedure, you can cancel your setting and go back anytime using the `Escape` button.

During the procedure, if you do not push any buttons for a long time, your settings are discarded and the display goes back to idle mode.

## main menu


In idle mode, press `Enter` to enter the main menu. Use the `Previous` or `Next` button to cycle through the menu items. They have the following meaning: 
* `J1`: immediate mode timer for relay 1
* `J2`: immediate mode timer for relay 2
* `J3`: immediate mode timer for relay 3
* `J4`: immediate mode timer for relay 4
* `S1 1`: first scheduled entry for relay 1
* `S1 2`: second scheduled entry for relay 1
* `S1 3`: third scheduled entry for relay 1
* `S2 1`: first scheduled entry for relay 2
* ...
* `S4 3`: third scheduled entry for relay 4

## switching on a relay immediately

You can activate a relay immediately for a limited amount of time. In the main menu, navigate to the desired item `J1` ... `J4`, then press `Enter`. The display shows the remaining time of the immediate mode timer for the chosen relays. `AUS` means "off". As another example, `18M` means "18 minutes remaining". Note that unfortunately `H` and `M` look very similar on the LED display.

To set the timer, press `Enter`. The display starts blinking to indicate that you can now change the value. Use the `Previous` and `Next` buttons for that. The following durations are available:
* `30S`: 30 seconds
* `60S`
* `02M`: 2 minites
* `03M`
* `05M`
* `10M`  
* `30M`
* `60M`
* `02H`: 2 hours
* `03H`
* `06H`
* `12H`
* `24H`: 24 hours 

Confirm your choice with `Enter`. As a confirmation, the display blinks three times quickly. 

The relay may react to your change with a slight delay. Reason: the need to change a relay state is only checked every 2 seconds or so. 

## making an entry into the schedule 

For every relay, there are three time slots available where it can be programmed to be switched on for a given duration. 

For example, to program time slot 2 of relay 1, choose `S1 2` in the main menu and press `Enter`. You are now in a sub menu with the following items:
* `DAU` (for German "Dauer"): duration
* `STA` : start time

To set the duration, choose `DAU` and hit `Enter`. The display shows the current setting for the duration. Press `Enter` again. The display starts blinking and you can set a duration in much the same way as in immediate mode. Pressing `Enter` again confirms the setting and takes you back to idle mode.

To set the start time, select `S1 2` in the main menu once more, and chose `STA` this time. You are now shown the current setting for the start time. When you press `Enter` again, the digits of the hour start blinking. Set the start time in the same way as you set the main clock in idle mode. 

Make sure both `DAU` and `STA` are configured to your satisfaction. When the clock time reaches the configured start time, you will notice that the relay switches on, for the configured duration.






