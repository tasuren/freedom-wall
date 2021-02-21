# Win32API
import win32gui
import win32con



# アクティブウィンドウを取得する。
def GetActiveWindow():
    # アクティブになっているウィンドウのハンドルを取得する。
    x = win32gui.GetForegroundWindow()
    # ハンドルとウィンドウのハンドルからゲットしたウィンドウタイトルをreturnする。
    return x,win32gui.GetWindowText(x)

# ウィンドウサイズの取得
def GetWindowRect(handle):
    return win32gui.GetWindowRect(handle)

# ウィンドウを名前から取得する。
def GetWindow(name,mode="all"):
    rt = []
    win32gui.EnumWindows(lambda x,p:rt.append([x,win32gui.GetWindowText(x)]),0)
    for x,n in rt:
        if mode == "all":
            if n == name:
                return x
        else:
            if name in n:
                return x
    return None

# ウィンドウをクリック貫通するようにする。
def setClickthrough(hwnd):
    try:
        styles = win32con.WS_EX_LAYERED | win32con.WS_EX_TRANSPARENT
        win32gui.SetWindowLong(hwnd,win32con.GWL_EXSTYLE,styles)
    except Exception as e:
        print(e)


# メッセージボックス
def MessageBoxInfo(title,desc):
    win32gui.MessageBox(0,desc,title,0x00000040)


# 動作テスト用
if __name__ == "__main__":
    print(GetWindow("ffdsahfdsa","bubun"))