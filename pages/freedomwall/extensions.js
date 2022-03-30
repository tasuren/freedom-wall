//! FreedomWall.js - Extensions

import { POST, request } from "./utils.js";


/**
 * Get all extension data.
 * The following is the data format of the extension
 * ```js
 * {
 *     "description": "Extension description",
 *     "author": "Extension author",
 *     "version": "Extension version",
 *     "setting": {
 *         "Key": "Value" // Extension's setting
 *     }
 * }
 * ```
 * @param {function} callback - Callback to be passed data (`{"ExtensionName": Above data}`)
 */
export function getExtensions(callback) {
    request(POST, "extensions/all/get", "", response => response.json().then(callback));
};


/**
 * Get extension data.
 * @param {string} name - Extension name
 * @param {function} callback - Callback to be passed data
 */
export function getExtension(name, callback) {
    request(POST, `extensions/one/get/${name}`, "", response => response.json().then(callback));
};


/**
 * Save extension setting.
 * @param {string} name - Extension name
 * @param {*} data - Extension setting data
 * @param {*} callback - Callback to be called
 */
export function updateExtensionSetting(name, data, callback) {
    request(POST, `extensions/one/update/${name}`, data, _ => callback());
};


/**
 * Reload extensions
 */
export function reloadExtensions() {
    request(POST, "extensions/reload/get", "", _ => { location.reload(); });
};