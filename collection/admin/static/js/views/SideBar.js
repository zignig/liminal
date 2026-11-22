// Sidebar component
// Needs stacked menu sets 

class SideBar {
    constructor(vnode) {
        this.items = [
            { name: "Home", ref: "#!/list" },
            { name: "Catalogue", ref: "#!/catalogue" },
            { name: "Search", ref: "#!/search" }
        ];
    }
    view(vnode) {
        return m("ul", this.items.map(item => m(SidebarItem, item)));
    };
}

class SidebarItem {
    constructor(vnode) {
        this.name = vnode.attrs.name;
        this.ref = vnode.attrs.ref;
    }
    view(vnode) {
        return m("li", m("a", { href: this.ref }, this.name))
    }
}