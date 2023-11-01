async function handleResponses(app) {
    while (true) {
        let message = await invoke("receive_response");
        console.log(message);
        app.ports.receiveResponse.send(message);
    }
}

function initBackend(app) {
    app.ports.sendCall.subscribe(call => invoke("handle_message", { call }));
    handleResponses(app);
}
