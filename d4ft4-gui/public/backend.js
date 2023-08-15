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
    addFunction(app, "add", ({ a, b }) => ({ a, b }), "callAdd", "returnAdd");
    addFunction(app, "server", ({ password, message }) => ({ password, message }), "callServer", "returnServer");
    addFunction(app, "client", ({ password, message }) => ({ password, message }), "callClient", "returnClient");
}