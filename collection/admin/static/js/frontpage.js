// Applcation data.
var interface = document.getElementById("interface")

// prefill the data
m.request(
    {
        method: "GET",
        url: "api/prefill",
    }
).then(function (dat) {
    console.log(dat);
}).catch(function (e) {
    console.log(e)
});

m.route(interface, "/list", {
    "/list": WorkList
})
