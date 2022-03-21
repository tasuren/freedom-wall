//! FreedomWall.js - Templates

import { request, POST } from "./utils.js";


/**
 * Get a list of names of templates that exist.
 * @param {function} callback - Callback will be passed templates.
 */
export function getTemplates(callback) {
    request(POST, "templates/all/get", "", response => response.json().then(callback));
};