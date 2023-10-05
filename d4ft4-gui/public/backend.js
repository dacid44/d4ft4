async function handleResponses(app) {
    while (true) {
        app.ports.receiveResponse.send(
            await invoke("receive_response")
        );
    }
}

function initBackend(app) {
    app.ports.sendCall.subscribe(call => invoke("handle_message", { call }));
    handleResponses(app);
}