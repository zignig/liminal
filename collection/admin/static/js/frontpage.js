// Applcation data.
// mithril application

var interface = document.getElementById("interface")

m.route(interface, "/list", {
    "/list": WorkList,
    "/catalogue" : Catalogue,
})
