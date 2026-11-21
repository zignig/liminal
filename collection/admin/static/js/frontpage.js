// Applcation data.
// mithril application

var interface = document.getElementById("interface")

// mount the side bar as a separate layout
m.mount(document.getElementById("sidebar"),SideBar);

m.route(interface, "/list", {
    "/list": WorkList,
    "/catalogue" : Catalogue,
})
