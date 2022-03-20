//! FreedomWall.js - Utils

var alertThrow = false;
const SILENT = () => {};
const POST = "POST";


/**
 * Do request.
 * @param {string} method - Method. Most of the time, `POST` is fine.
 * @param {string} endpoint - Request Destination. Example: `setting/language/get`.
 * @param {Object} body - Data to be sent.
 * @param {function(Response)} callback - response will be passed to this.
 */
 export function request(method, endpoint, body, callback) {
    if (endpoint.indexOf("reply") === -1) {
        throw "This endpoint is not available.";
    };
    let isString = typeof(body) == "string" || body instanceof String;
    return await fetch(new Request(`wry://api/${endpoint}`, {
        method: method,
        header: {"Content-Type": isString ? "text/plain" : "application/json"},
        body: isString ? body : JSON.stringify(body)
    }))
        .then(response => response.text())
        .then(_ => {
            // レスポンスを待機する。
            var ok = false;
            for (let i = 1; i < 10; i++) {
                setTimeout(() => {
                    if (!ok) {
                        fetch(new Request("wry://api/reply"))
                            .then(response => {
                                if (response.status != 503) {
                                    ok = true;
                                    response.text().then(text => {
                                        if (response.status == 400 || response.status == 404) {
                                            if (alertThrow) { alert(text); };
                                            throw text;
                                        };
                                    })
                                    callback(response);
                                };
                            });
                    };
                }, 1000 * i);
                if (!ok) { break; };
            };
        });
};