# FreedomWall (under development now)
This application allows you to add wallpaper to any application.  
As for Discord of chat app, you can add wallpaper without violating the Terms of Service.
(The reason for this is explained in the `How` below.)  
Currentrly, Windows and MacOS are supported.
(Linux will be supported at some point, as indicated in [this issue](https://github.com/tasuren/FreedomWall/issues/14).)  

## Supported Files
* Picture (All image formats supported by WebView on your OS)
* Video (All video formats supported by WebView on your OS)
* Website (YouTube, of course)

## How
FreedomWall does the simple thing of displaying a translucent window with a background on top of the app you want to set the background for.  
Therefore, you can set wallpaper safely without modifying any application.  
**So you can wallpaper your Discord without violating the Terms of Service.**  
It also uses WebView to depict the wallpaper so that websites can be displayed.

## Screenshots
Coming soon...

## Build
We need Rust's Cargo, so let's prepare that first.  
Once you have it ready, simply run `cargo build --release` at the top of this repository directory.