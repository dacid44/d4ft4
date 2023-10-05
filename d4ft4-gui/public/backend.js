function returnQueue(callPort, returnPort) {
    if (!(callPort in app.ports))
        throw new Error(`Missing call port: ${callPort}`);
    if (!(returnPort in app.ports))
        throw new Error(`Missing return port: ${returnPort}`);

    let nextCallId = 0;
    let nextReturnId = 0;
    let waitingResponses = new Map();

    function nextId() {
        return nextCallId++;
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
        console.log("call", name, id, value);
        invoke(name, args(value))
            .then(value => {
                console.log("response", name, id, value);
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

async function handleResponses(app) {
    while (true) {
        app.ports.receiveResponse.send(
            await invoke("receive_response")
        );
    }
}

function initBackend(app) {
    // addFunction(app, "handle_message", (call) => ({ call }), "sendCall", "receiveResponse");

    // app.ports.callSelectFile.subscribe(({ connId, save }) => {
    //     (save ? dialog.save() : dialog.open({ directory: false, multiple: false }))
    //         .then(path => app.ports.returnSelectFile.send([connId, path]));
    // })
    // addFunction(app, "open_file_dialog", (save) => ({ save }), "callOpenFileDialog", "returnOpenFileDialog");
    app.ports.sendCall.subscribe(call => invoke("handle_message", { call }));
    handleResponses(app);
}