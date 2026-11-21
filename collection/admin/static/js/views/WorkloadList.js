var WorkList = function () {
    var overlay = function () {
        var self = this;
        var data = [];

        self.oninit = function () {
            m.request(
                {
                    method: "GET",
                    url: "api/worklist",
                }
            ).then(function (wl) {
                data = []
                wl.map(workload => {
                    wl = WorkRow(workload);
                    data.push(wl);
                })
                // console.log(data);
            });
        }

        self.controller = function () { }

        self.view = function (ctrl, player) {
            return m("table", {
                class: "table is-fullwidth is-striped",
            },
                m("tbody",
                    data.map(workload => m(workload))
                )
            );
        }
        return self
    }
    return new overlay()
};

function WorkRow(workload) {
    var row = function (workload) {
        var self = this;
        self.name = workload.name;
        self.id = workload.id;
        self.status = workload.status

        function Sender(workload) {
            m.request(
                {
                    method: "GET",
                    url: "api/worklist/:item/:action",
                    params: {
                        item: workload.name,
                        action: workload.status
                    }
                }
            ).then(function (dat) {
                console.log(dat);
            }).catch(function (e) {
                console.log(e)
            });
        }

        self.view = function () {
            var cl;
            var bt;
            var new_status;
            switch (self.status) {
                case "Running":
                    cl = 'is-primary'
                    bt = "stop"
                    new_status = "Stopped"
                    break;
                case "Stopped":
                    cl = 'is-warning'
                    bt = "start"
                    new_status = "Running"
                    break;
            };
            return m("tr", { class: cl },
                [
                    m("td", self.name),
                    m("td", self.status),
                    m("td", m("a", {
                        class: "button is-small",
                        onclick: () => {
                            self.status = new_status;
                            Sender(self);
                        }
                    },
                        bt))
                ]
            )

        }
    }
    return new row(workload)
}