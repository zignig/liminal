// const eventSource = new EventSource("events");

// eventSource.onopen = () => {
//     document.getElementById("status").textContent = "Connected";
// };

// Set up the form handler.
console.log("load app.js");
let prefill_click = document.getElementById("prefill");

prefill_click.addEventListener("click", (e) => {
    e.preventDefault();
    console.log("click prefill button");
    m.request(
        {
            method: "GET",
            url: "api/prefill",
        }
    ).then(function (wl) {
    });

});
