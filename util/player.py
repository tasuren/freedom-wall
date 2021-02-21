# FreedomWall - 壁紙プレイヤー

from cv2 import VideoCapture,cvtColor,COLOR_BGR2RGB,CAP_PROP_POS_FRAMES,CAP_PROP_FRAME_WIDTH,CAP_PROP_FRAME_HEIGHT,CAP_PROP_FPS
from tkinter import messagebox
from PIL import Image,ImageTk



class TkPlayer():
    def __init__(self,path):
        self.path = path
        # 未設定なら以降実行しない。
        if self.path == "":
            self.fps = 1
            return

        if path[-3:] in ["mp4","avi"]:
            self.mode = "video"
            self.video = VideoCapture(path)

            if not self.video.isOpened():
                self.path = ""
                messagebox.showwarning("FreedomWall",f"{path}\nのファイルが開けないため壁紙を貼り付ける事ができませんでした。")
                return

            self.height = self.video.get(CAP_PROP_FRAME_HEIGHT)
            self.width = self.video.get(CAP_PROP_FRAME_WIDTH)
            self.fps = self.video.get(CAP_PROP_FPS)
        else:
            self.mode = "picture"
            try:
                self.picture = Image.open(path)
            except:
                self.path = ""
                messagebox.showwarning("FreedomWall",f"{path}\nのファイルが開けないため壁紙を貼り付ける事ができませんでした。")
                return
            self.height,self.width = self.picture.size
            self.fps = 1

    # 現在のフレームの取得
    def get_frame(self,height,width):
        # もしpathが空なら実行しない。
        if self.path == "":
            return None
        
        # 写真だったら写真で出力。
        if self.mode == "picture":
            return ImageTk.PhotoImage(image=self.picture.resize((width,height)))

        # 動画なら動画で出力。
        if self.video.isOpened():
            ret,frame = self.video.read()

            # 最後だったら最初からにしてもう一度読み込む。
            if not ret:
                self.video.set(CAP_PROP_POS_FRAMES,0)
                ret,frame = self.video.read()

            if ret:
                return ImageTk.PhotoImage(image=Image.fromarray(cvtColor(frame,COLOR_BGR2RGB)).resize((width,height)))
            else:
                return None
        
        else:
            return None

    # 解放
    def __del__(self):
        if self.path == "":
            return
        if self.mode == "video":
            if self.video.isOpened():
                self.video.release()