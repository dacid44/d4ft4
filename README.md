# D4FT4

## To Do:
- [ ] Basic text transfer
- [ ] Single file transfer
- [ ] 


How multiple file transfer will work:
- Receiver gets a message saying the top level received file, with the count of files and directories, and the expected total size
- Receiver has option to see full list (or browse? maybe for future?)
- Receiver chooses where to save the file (defaults to current directory or home directory and original name of file or folder)
    - if the chosen path is a folder that already exists, save it in that folder instead
    - in GUI, don't allow choosing a folder if being sent a file
    - if being sent multiple items, then expect/prompt for the user to choose a (single) folder that already exists
- send accept/reject (at least for now, accept/reject is all or nothing)
- transfer, etc...


- Sender sends the file list
- Receiver sends back the list of accepted files
- Sender sends only those files
- Directories only exist in the file list



Brainstorming for UI:
- Top section divided up into two parts, for text and files. Once you start a transfer, one will expand to take up the whole space.
- Old version did something like this, had sliding panes to go between them
- Bottom section is everything not dependent on what kind of transfer
- Text side has incoming text like in chat bubbles maybe?
- Could actually do the same thing for files
- Oh! could structure like an email program: have a "send" button which brings up a tabbed interface with to send either text or files.
- Have a way to go to a "receive" screen
  - have an option to choose between receiving text, files, or autodetect
  - also have a way to choose between connecting and listening
  - this component could be common with the sending page but maybe not
  - airdrop-like screen, where you view an incoming message, and if it's a file, select where to save it
- Have a way to view history
  - copy past text received
  - open explorer to past files received
  - have a way to delete old received stuff from log/stop tracking it
  - maybe have a way to see sent history? but doesn't make quite as much sense as receive history