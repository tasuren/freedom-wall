//! FreedomWall.js - Utils


export const SILENT = () => {};
export const POST = "POST";
export const GET = "GET";
var request_count = 0;
window.__callbacks__ = {};


let params = (new URL(location)).searchParams;
if (params.has("window_id")) window.__WINDOW_ID__ = params.get("window_id");
window.addEventListener("load", _ => {
    for (let element of document.getElementsByTagName("a"))
        element.setAttribute("href", `${element.getAttribute("href")}?window_id=${window.__WINDOW_ID__}`);
});


/**
 * Do request.
 * @param {string} method - Method. Most of the time, `POST` is fine.
 * @param {string} endpoint - Request Destination. Example: `setting/language/get`.
 * @param {Object} body - Data to be sent.
 * @param {function} callback - response data (Json/String) will be passed to this.
 * @param {boolean} isResponseJson - Is response json
 * @param {reload} reload - Whether do reload
 * @param {boolean} doAlert - Whether do alert on error
 */
export function request(method, endpoint, body, callback, isResponseJson=false, reload=undefined, doAlert=true) {
    if (endpoint.indexOf("reply") !== -1) {
        throw "This endpoint is not available.";
    };

    if (request_count > 10) { request_count = 0; };
    request_count += 1;
    // リロードするかどうか。もし指定されなかった場合はupdate時のみ自動でリロードする。
    reload = typeof reload === "undefined" ? endpoint.indexOf("update") !== -1 : reload;
    // もしリロードするかつローディングが実装されているのならローディングを表示する。
    if (reload && window.loadingShow) window.loadingShow();
    // bodyが文字列かどうかをチェックする。
    let isString = typeof body == "string" || body instanceof String;

    // コールバックを設定する。
    window.__callbacks__[request_count] = function (status, data) {
        scrollTo(0, 0);
        if (reload && window.loadingShow) window.loadingShow();
        if (status < 400) {
            callback(isResponseJson ? JSON.parse(data) : data);
            if (typeof reload == "string" || reload instanceof String)
                location = reload
            else if (reload) location.reload();
        } else {
            if (window.loadingShow) {
                window.loadingSetText(data);
                window.loadingShow();
            };
            throw data;
        };
    };

    // リクエストを行う。
    
    console.log(`Request[Method:${method},RequestId:${request_count},Reload:${reload}] ${endpoint}`);
    fetch(new Request(`__SCHEME__api/${window.__WINDOW_ID__}/${request_count}/${endpoint}`, {
        method: method,
        header: {"Content-Type": isString ? "text/plain" : "application/json"},
        body: isString ? body : JSON.stringify(body)
    }))
        .then(response => response.text())
};


/**
 * Open file dialog
 * @param {function} callback - Callback to be passed path
 */
export function open(callback) {
    request(POST, "open/.../...", "", SILENT, false, false);
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
 * Open website
 * @param {string} path - Path to folder
 * @param {function} callback - Callback to be called when opened
 */
 export function openWebsite(path, callback) {
    request(POST, "openWebsite/.../...", path, (_) => callback());
};


/**
 * Get the path to the FreedomWall configuration folder.
 * @param {function} callback - Callback to be passed path
 */
export function getPath(callback) {
    request(POST, "getPath/.../...", "", callback);
};