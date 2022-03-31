//! FreedomWall.js - Utils

import { setInterval } from "./setting.js";


export const SILENT = () => {};
export const POST = "POST";


/**
 * Do request.
 * @param {string} method - Method. Most of the time, `POST` is fine.
 * @param {string} endpoint - Request Destination. Example: `setting/language/get`.
 * @param {Object} body - Data to be sent.
 * @param {function(Response)} callback - response will be passed to this.
 */
export function request(method, endpoint, body, callback, interval=30, reload=true, doAlert=true) {
    if (endpoint.indexOf("reply") !== -1) {
        throw "This endpoint is not available.";
    };
    let doReload = reload && endpoint.indexOf("update") !== -1;
    console.log(`Request[${method},${reload}] ${endpoint}`);
    if (doReload && window.loadingShow) window.loadingShow();
    let isString = typeof(body) == "string" || body instanceof String;
    fetch(new Request(`https://wry.api/${endpoint}`, {
        method: method,
        header: {"Content-Type": isString ? "text/plain" : "application/json"},
        body: isString ? body : JSON.stringify(body)
    }))
        .then(response => response.text())
        .then(_ => {
            // レスポンスを待機する。
            var ok = false;
            for (let i = 1; i < 50; i++) {
                setTimeout(() => {
                    if (!ok) {
                        fetch(new Request("https://wry.api/reply"))
                            .then(response => {
                                if (response.status != 503) {
                                    ok = true;
                                    if (response.status == 400 || response.status == 404)
                                        response.text().then(text => {
                                            if (doAlert) {
                                                window.loadingSetText(`Error:\n${text}`);
                                                window.loadingShow();
                                            };
                                            throw text;
                                        })
                                    else {
                                        callback(response);
                                        if (doReload) {
                                            scrollTo(0, 0);
                                            location.reload();
                                        };
                                    };
                                };
                            })
                    };
                }, interval * i);
                if (ok) { break; };
            };
        });
};


/**
 * Open file dialog
 * @param {function} callback - Callback to be passed path
 */
export function open(callback) {
    request(POST, "open/.../...", "", SILENT, 10, false);
    window._fileSelected = callback;
};


/**
 * Open folder
 * @param {string} path - Path to folder
 * @param {function} callback - Callback to be called when opened
 */
export function openFolder(path, callback) {
    request(POST, "openFolder/.../...", path, (_) => callback());
};


/**
 * Get the path to the FreedomWall configuration folder.
 * @param {function} callback - Callback to be passed path
 */
export function getPath(callback) {
    request(POST, "getPath/.../...", "", (response) => response.text().then(callback));
};