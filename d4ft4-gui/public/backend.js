function addFunction(app, name, args, callPort, returnPort) {
    let nextCallId = 0;
    let nextReturnId = 0;
    let waitingResponses = new Map();

    function returnWaiting() {
        while (waitingResponses.has(nextReturnId)) {
            app.ports[returnPort].send(waitingResponses.get(nextReturnId));
            waitingResponses.delete(nextReturnId);
            nextReturnId++;
        }
    }

    if (!(callPort in app.ports))
        throw new Error(`Missing call port: ${callPort}`);
    if (!(returnPort in app.ports))
        throw new Error(`Missing return port: ${returnPort}`);

    app.ports[callPort].subscribe(value => {
        const id = nextCallId++;
        invoke(name, args(value))
            .then(value => {
                if (id == nextReturnId) {
                    app.ports[returnPort].send(value);
                    nextReturnId++;
                    returnWaiting();
                } else {
                    waitingResponses.set(id, value);
                }
            });
    });
}

function initBackend(app) {
    addFunction(app, "setup", ({ connId, isServer, mode, password }) => ({ connId, isServer, mode, password }), "callSetup", "returnMessage");
    addFunction(app, "send_text", ({ connId, text }) => ({ connId, text}), "callSendText", "returnMessage");
    addFunction(app, "receive_text", ({ connId }) => ({ connId }), "callReceiveText", "returnMessage");
    addFunction(app, "send_file", ({ connId, path }) => ({ connId, path }), "callSendFile", "returnMessage");
    addFunction(app, "receive_file", ({ connId, path }) => ({ connId, path }), "callReceiveFile", "returnMessage");

    app.ports.callSelectFile.subscribe(({ connId, save }) => {
        (save ? dialog.save() : dialog.open({ directory: false, multiple: false }))
            .then(path => app.ports.returnSelectFile.send([connId, path]));
    })
}