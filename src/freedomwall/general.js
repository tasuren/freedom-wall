//! FreedomWall.js - General

import { request, SILENT, POST } from "./utils.js";


/**
 * Update language setting.
 * @param {string} language - Language code
 */
export function postLanguage(language) {
    request(POST, "setting/language/update", language, SILENT);
};


/**
 * Get a language setting.
 * @param {function(string)} - Callback will be passed a language code.
 */
export function getLanguage(callback) {
    request(
        POST, "setting/language/get", "", response,
        response => response.text().then(callback)
    );
}