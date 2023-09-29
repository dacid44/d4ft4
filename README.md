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
