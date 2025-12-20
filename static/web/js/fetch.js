// frontpage conversion to unpoly

// Set up the form handler.
let blobForm = document.getElementById('blob-upload');
let messageField = blobForm.querySelector("#blobtext");

blobForm.addEventListener("submit", (e) => {
    e.preventDefault();

    const message = messageField.value;
    fetch("/blob", {
        method: "POST",
        body: new URLSearchParams({ message }),
    }).then((response) => {
        console.log(response.body);
        if (response.ok) {
            messageField.value = "";
            messageField.className = "textarea is-success";
        }
    });

})