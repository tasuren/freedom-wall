# Setting Wallpaper to Application
Finally, apply the desired wallpaper to the application.  

![Settings Wallpaper](https://user-images.githubusercontent.com/45121209/161428752-5eb0fa40-5b3e-4990-9439-ef934bc93522.png)  

Click `Setting` on the left menu to go to the setting screen.  
Select the wallpaper you created earlier from the `Select Box` in `Add: Select Box`.  
This adds a setting named `nothing`.  
Click it to go to the detailed setting screen.  

## Target
Enter the text contained in the title of the application window to which you want to add the wallpaper.  
On a Mac, enter the text found in the application name, not the window title.  
Multiple specifications can be specified by using commas (`,`).

## Exception
Exception to set no wallpaper.  
You will not set wallpaper for windows that contain the characters you type.  
For example, let's say you wallpaper an app called Discord, and then use your browser to view Discord-related web pages.  
And suppose the title of your web page contains `Discord`.  
If the browser puts the title in the window title, the wallpaper is also set in the browser.  
This configuration is to deal with this.  
Like the Target setting, you can specify multiple targets by commas.

## Transparency
As described in `How` on the first page, the software simulates the translucent wallpaper by displaying a window on top of the application.  
This is its transparency setting.

## Adjustment
You can adjust the size of this setting.  
You can change the display position or size of the wallpaper.  
This is useful if the window has a special shape or you want to display partial wallpaper.  
Raise a value of `left 'or `right' if you want to shift left and right.  
For example, if you want to shift to the right, you can increase the value of `left` to put a space on the left.  
If you want to move up or down, increase the value of `up` or `down`.  
For example, if you want to shift it down, you can increase `up` to put a space above it.  
Of course, negative values are also supported.  
That is, it can shrink.
### Example
In this example, the wallpaper is applied only to the black areas in the terminal.
#### Before
The wallpaper is also reflected in the bar and tabs above.  
Depending on the wallpaper, it may be noticeable.  
![Adjust Example Before](https://user-images.githubusercontent.com/45121209/161429973-8268849b-da4f-4224-9419-d4588f6beaa0.png)
#### After
Fixed  
By this setting: `up: 55, down: -55`  
![Adjust Example After](https://user-images.githubusercontent.com/45121209/161429970-cbe20f05-194d-49f7-a9eb-86875b57d662.png)

## Save / Delete
It is possible with the two buttons at the bottom.