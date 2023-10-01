//! FreedomWall.js - Wallpapers

import { request, SILENT, POST } from "./utils.js";


/**
 * Get a list of available wallpapers.
 * The format of the wallpaper data is as follows:
 * ```js
 * {
 *     "author": "Author",
 *     "description": "Description",
 *     "setting": {}, // Value passed as a query parameter to the wallpaper URL when loading the wallpaper.
 *     "force_size": true // Whether to force the width and height of the style of the element with the background class to the window size.
 * }
 * ```
 * @param {function} callback - Callback to be passed wallpapers.
 */
export function getWallpapers(callback) {
    request(POST, "wallpapers/all/get", "", callback, true);
};


/**
 * Register a new wallpaper profile from the template.
 * @param {string} template - Template name
 * @param {string} name - Wallpaper profile name
 */
export function postWallpaper(template, name) {
    request(POST, "wallpapers/all/update", `${template}?${name}`, SILENT);
};


function replaceSlash(text) { return text.replaceAll("/", "-"); };


/**
 * Change the wallpaper profile settings.
 * @param {string} name - Wallpaper Profile Name
 * @param {object} data - Wallpaper Data / `null` / after name
 * @param {string} mode - `update` / `remove` / `rename`
 */
export function updateWallpaper(name, data, mode, callback=SILENT, reload=true) {
    if (mode == "rename") {
        request(
            POST, `wallpapers/rename/update/${replaceSlash(name)}/${replaceSlash(data)}`,
            "", callback, false, reload
        )
    } else {
        request(
            POST, `wallpapers/one/update/${replaceSlash(name)}/${mode}`,
            data ? JSON.stringify(data) : "", callback, false, reload
        );
    };
};