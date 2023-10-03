function returnQueue(callPort, returnPort) {
    if (!(callPort in app.ports))
        throw new Error(`Missing call port: ${callPort}`);
    if (!(returnPort in app.ports))
        throw new Error(`Missing return port: ${returnPort}`);

    let nextCallId = 0;
    let nextReturnId = 0;
    let waitingResponses = new Map();

    function nextId() {
        return nextReturnId++;
    }

    function returnWaiting() {
        while (waitingResponses.has(nextReturnId)) {
            app.ports[returnPort].send(waitingResponses.get(nextReturnId));
            waitingResponses.delete(nextReturnId);
            nextReturnId++;
        }
    }

    function handleResponse(id, value) {
        if (id == nextReturnId) {
            app.ports[returnPort].send(value);
            nextReturnId++;
            returnWaiting();
        } else {
            waitingResponses.set(id, value);
        }
    }

    return { nextId, handleResponse }
}

function addFunction(app, name, args, callPort, returnPort) {
    let queue = returnQueue(callPort, returnPort);

    app.ports[callPort].subscribe(value => {
        const id = queue.nextId();
        invoke(name, args(value))
            .then(value => {
                queue.handleResponse(id, value);
            });
    });
}

// returns a Result from the command
function addFallibleFunction(app, name, args, callPort, returnPort) {
    let queue = returnQueue(callPort, returnPort);

    app.ports[callPort].subscribe(value => {
        const id = queue.nextId();
        invoke(name, args(value))
            .then(value => {
                queue.handleResponse(id, { Ok: value })
            })
            .catch(error => {
                queue.handleResponse(id, { Err: error })
            })
    })
}

function initBackend(app) {
    addFunction(app, "setup", ({ connId, address, isServer, mode, password }) => ({ connId, address, isServer, mode, password }), "callSetup", "returnSetup");
    addFunction(app, "send_text", ({ connId, text }) => ({ connId, text}), "callSendText", "returnSendText");
    addFunction(app, "receive_text", ({ connId }) => ({ connId }), "callReceiveText", "returnReceiveText");
    // addFunction(app, "send_file", ({ connId, path }) => ({ connId, path }), "callSendFile", "returnSendFile");
    // addFunction(app, "receive_file", ({ connId, path }) => ({ connId, path }), "callReceiveFile", "returnReceiveFile");

    // app.ports.callSelectFile.subscribe(({ connId, save }) => {
    //     (save ? dialog.save() : dialog.open({ directory: false, multiple: false }))
    //         .then(path => app.ports.returnSelectFile.send([connId, path]));
    // })
}