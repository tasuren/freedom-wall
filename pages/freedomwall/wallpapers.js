//! FreedomWall.js - Wallpapers

import { request, SILENT, POST } from "./utils.js";


export function getWallpapers(callback) {
    request(POST, "wallpapers/all/get", "", response => response.json().then(callback));
}