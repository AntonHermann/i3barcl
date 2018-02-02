* run program as Command from the program, restarting it if it aborts (if 'broken_pipe' errors keep occuring)
* improve error handling, introducing custom error types
* add logging to a file (since stderr on the screen conflicts with the ncurses interface
* fix display errors with unicode icons
* support full set of input properties
  - color
  - background
  - min-width (at least the string syntax)
  - urgent
  - separator
  - markup (at least some of it like bold, ...)
* (in the far future: support click events of the blocks)
* add comprehensive testing
