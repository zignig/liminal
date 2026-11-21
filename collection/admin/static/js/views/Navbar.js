var NavBar = function () {
    var overlay = function () {
        var self = this;
        var items = ["/", "workloads", "catalogue", "search"]
        self.oninit = function () {
        }
        self.view = function (ctrl, player) {
            return m("tabs",
                {
                    class: "tabs is-boxed"
                },
                m("ul", items.map(item => m("li", m("a", { href: item }, item)))
                )
            );
        };

        return self
    }
    return new overlay()
};