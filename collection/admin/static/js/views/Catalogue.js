var Catalogue = function () {
    var overlay = function () {
        var self = this;
        var data = [];

        self.oninit = function () {
            console.log("Catalogue");
            // should fetch the catalogue from network
            data.push(CatalogueItem("bob"));
            data.push(CatalogueItem("bill"));
            data.push(CatalogueItem("bradley"));
        }

        self.controller = function () { }

        self.view = function () {
            return m("div", { class: "container" },
                [
                    m("h2", "Catalogue"),
                    data.map(item => m("section", { class: "section" },
                        m(item)
                    )),
                ]);
        }
        return self
    }

    return new overlay()
};


var CatalogueItem = function (name) {
    var overlay = function (name) {
        var self = this;
        self.view = function () {
            return m("article", { class: "card" },
                [
                    m("header",
                        [
                            m("h3", name),
                            m("p",
                                "description"
                            ),
                        ],
                    ),
                    m("p", "stuff stuff stuff"),
                ]
            )
        }
        return self
    }
    return new overlay(name)
}