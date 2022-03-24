//! FreedomWall.js - Utils

import { setInterval } from "./setting.js";


export const alertThrow = true;
export const afterReload = true;
export const SILENT = () => {};
export const POST = "POST";


/**
 * Do request.
 * @param {string} method - Method. Most of the time, `POST` is fine.
 * @param {string} endpoint - Request Destination. Example: `setting/language/get`.
 * @param {Object} body - Data to be sent.
 * @param {function(Response)} callback - response will be passed to this.
 */
export function request(method, endpoint, body, callback, interval=1000, reload=true) {
    if (endpoint.indexOf("reply") !== -1) {
        throw "This endpoint is not available.";
    };
    let doReload = reload && afterReload && endpoint.endsWith("update");
    if (doReload && window.loadingShow) window.loadingShow();
    let isString = typeof(body) == "string" || body instanceof String;
    fetch(new Request(`wry://api/${endpoint}`, {
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
                                    if (response.status == 400 || response.status == 404)
                                        response.text().then(text => {
                                            if (alertThrow) { alert(text); };
                                            throw text;
                                        });
                                    else callback(response);
                                    if (doReload) {
                                        scrollTo(0, 0);
                                        location.reload();
                                    };
                                };
                            })
                    };
                }, interval * i);
                if (ok) { break; };
            };
        });
};


export function startInteraction(callback) { setInterval(0.001, callback); };
export function endInteraction() { setInterval("setting"); };