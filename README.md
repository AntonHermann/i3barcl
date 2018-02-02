# i3barcl - a command line viewer for content in the i3bar format
## this is a tool for simple testing of i3status-like scripts without the need to change your i3 config and restarting your whole i3 just to test some changes

Simply pipe the output of i3status or any other tool that formats its output
according to the [i3bar protocol](https://i3wm.org/docs/i3bar-protocol.html)

i3barcl outputs the data using ncurses, refreshing on change, like i3bar would
