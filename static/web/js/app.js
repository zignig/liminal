// Setup for the chat stuff

var STATE = {
    room: "lobby",
    rooms: {},
    connected: true,
}
// Set up the form handler.
let newMessageForm = document.getElementById('new-message');
let messageField = newMessageForm.querySelector("#message");
let usernameField = newMessageForm.querySelector("#username");

newMessageForm.addEventListener("submit", (e) => {
    e.preventDefault();

    const room = STATE.room;
    const message = messageField.value;
    const username = usernameField.value || "guest";
    if (!message || !username) return;

    if (STATE.connected) {
        fetch("/message", {
            method: "POST",
            body: new URLSearchParams({ room, username, message }),
        }).then((response) => {
            if (response.ok) messageField.value = "";
        });
    }
})