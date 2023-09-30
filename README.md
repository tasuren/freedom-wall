![GitHub all releases](https://img.shields.io/github/downloads/tasuren/FreedomWall/total) ![GitHub release (latest by date)](https://img.shields.io/github/v/release/tasuren/FreedomWall) [![Discord](https://img.shields.io/discord/777430548951728149?label=chat&logo=discord)](https://discord.gg/kfMwZUyGFG)
# FreedomWall
This application allows you to add wallpaper to any application.  
As for Discord of chat app, you can add wallpaper without violating the Terms of Service.
(The reason for this is explained in the `How` below.)  
Currentrly, Windows and MacOS are supported.
(Linux will be supported at some point, as indicated in [this issue](https://github.com/tasuren/FreedomWall/issues/14).)  

**Warning: This is early stage.**

![FreedomWall](https://user-images.githubusercontent.com/45121209/161414150-61a726fb-60be-4007-964a-448d62d6c60a.gif)

## Supported Files
* Picture (All image formats supported by WebView on your OS)
* YouTube
* HTML (It will be available soon, but can be created by placing html files in a folder.)

## How
FreedomWall does the simple thing of displaying a translucent window with a background on top of the app you want to set the background for.  
Therefore, you can set wallpaper safely without modifying any application.  
**So you can wallpaper your Discord without violating the Terms of Service.**  
It also uses WebView to depict the wallpaper so that websites can be displayed.

## Download
This can be downloaded [here](https://github.com/tasuren/FreedomWall/releases).  
[This guide](http://freedomwall.f5.si) is an easy way to learn how to use it.

## Build
It needs [Rust](https://www.rust-lang.org/tools/install), so let's prepare that first.  

### Windows
Simply run `cargo build --release`.

### Mac
Install cargo-bundle with `cargo install cargo-bundle` and then `cargo bundle --release`.  
The completed app file should be placed in the Applications folder and executed.  
At this time error messages will not display properly if you do not do so.

## Contributing
See `contributing.md` in the repository.

## Screenshots
![FreedomWall2](https://user-images.githubusercontent.com/45121209/161414647-ef6d405f-8edb-4ea1-b0fb-3e4414be1f80.gif)  
<img width="931" alt="FreedomWall Example LINE" src="https://user-images.githubusercontent.com/45121209/161413770-5b8616da-9509-4205-9f10-f62e52731a4f.png">
