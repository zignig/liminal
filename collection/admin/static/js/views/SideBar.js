var SideBar = function () {
    var overlay = function () {
        var self = this;
        var items = [
            { name: "Home", ref: "#!/list" },
            { name: "Catalogue", ref: "#!/catalogue" },
            { name: "Search", ref: "#!/search" }
        ];

        self.oninit = function () {
        }

        self.view = function (ctrl, player) {
            return m("ul", items.map(item => m(SidebarItem,item)));
        };

        return self
    }
    return new overlay()
};

class SidebarItem {
    constructor(vnode) {
        this.name = vnode.attrs.name;
        this.ref = vnode.attrs.ref;
    }
    view(vnode) {
        return m("li", m("a", { href: this.ref }, this.name))
    }
}