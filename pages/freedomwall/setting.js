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
 * @param {function} callback - Callback will be passed a language code.
 */
export function getLanguage(callback, interval=10) {
    request(
        POST, "setting/language/get", "",
        response => response.text().then(callback), interval
    );
};


/**
 * Update wallpaper settings.
 * Wallpaper setting data is in the following format
 * ```js
 * {
 *     "targets": [], // List of strings included in the name of the application to which the wallpaper will be attached
 *     "exceptions": [], // List of strings included in the name of the application to which the wallpaper will be not attached.
 *     "alpha": 0.2, // Transparency level
 *     "wallpaper": "" // The name of the wallpaper to be attached.
 * }
 * ```
 * @param {list} wallpapers - This is the list that contains the objects above.
 */
export function postWallpapers(wallpapers) {
    request(POST, "setting/wallpapers/update", wallpapers, SILENT);
};


/**
 * Get wallpaper settings.
 * The data to be passed is described in `postWallpapers`.
 * @param {function} callback - Callback will be passed wallpaper settings.
 */
export function getWallpapers(callback) {
    request(
        POST, "setting/wallpapers/get", "",
        response => response.text().then(callback)
    );
};


/**
 * Sets the interval for adjusting the background window.
 * @param {number} interval - Interval
 */
export function postInterval(interval) {
    request(POST, "setting/interval/post", interval.toString(), SILENT);
};


/**
 * Gets the interval to adjust the currently set backgrou
 * @param {function} callback - Callback will be passed interval.
 */
export function getInterval(callback) {
    request(
        POST, "setting/interval/get", "",
        response => response.text().then(interval => callback(parseFloat(interval)))
    );
};


/**
 * Set developer mode.
 * @param {boolean} onoff 
 */
export function postDev(onoff) {
    request(POST, "setting/dev/update", Number(onoff).toString(), SILENT);
};


/**
 * Get developer mode.
 * @param {function} callback - Callback will be passed whether developer mode is enabled or not.
 */
export function getDev(callback) {
    request(POST, "setting/dev/get", "", result => callback(Boolean(Number(result))));
};