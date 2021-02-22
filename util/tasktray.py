# FreedomWall - タスクトレイモジュール

from util.win32 import GetWindow

from tkinter import simpledialog,filedialog,messagebox
from infi.systray import SysTrayIcon

from json import load,dump
from PIL import Image


credit_text = """# Freedom Wall 1.1.1
開発言語：Python
開発者　：tasuren

Copyright (C) 2021 by tasuren

# 使用ライブラリ
## Pillow
Copyright (C) 1997-2011 by Secret Labs AB
Copyright (C) 1995-2011 by Fredrik Lundh
## OpenCV
Copyright (C) 2000-2020, Intel Corporation, all rights reserved.
Copyright (C) 2009-2011, Willow Garage Inc., all rights reserved.
Copyright (C) 2009-2016, NVIDIA Corporation, all rights reserved.
Copyright (C) 2010-2013, Advanced Micro Devices, Inc., all rights reserved.
Copyright (C) 2015-2016, OpenCV Foundation, all rights reserved.
Copyright (C) 2015-2016, Itseez Inc., all rights reserved.
Copyright (C) 2019-2020, Xperience AI, all rights reserved.
## Threading
Copyright (c) 2019 Loreto Parisi
## infi.systray
Copyright (c) 2017 INFINIDAT
## ライセンス
上記のライブラリのライセンスはFreedomWallフォルダのLICENSEフォルダにテキストファイルとして記載されています。"""


class TaskTray():
	def __init__(self,wallcord):
		self.window = wallcord
		self.root = self.window.root

		with open("data.json","r") as f:
			self.data = load(f)

		# メインスレッドじゃないとTkinterメソッドを実行できない。
		# だからメインスレッドのFreedomWallクラスのself.qに実行したいのを追加する。
		# そしてメインスレッドから実行する。
		# だから lambda がある。
		self.icon = SysTrayIcon(
			"icon.ico",
			"FreedomWall",
			(
				("FreedomWall",None,lambda sysTrayIcon: self.window.q.append(self.credit)),
				("Set",None,lambda sysTrayIcon: self.window.q.append(self.setting)),
				("Del",None,lambda sysTrayIcon: self.window.q.append(self.delete)),
				("List",None,lambda sysTrayIcon: self.window.q.append(self.setting_list))
			),
			on_quit=lambda sysTrayIcon: self.window.q.append(self.exit)
		)

	# 壁紙の設定。
	def setting(self):
		title = simpledialog.askstring("FreedomWall","設定したいウィンドウのタイトルにある文字を入力してください。")
		if not title:
			return
		alpha = simpledialog.askstring("FreedomWall","壁紙の透明度を入力してください。\nデフォルトは0.2です。\n元の背景が白の場合は0.3あたりの数値が良いです。\n元の背景が黒の場合は0.1あたりの数値が良いです。")
		if not alpha:
			alpha = 0.2
		try:
			alpha = float(alpha)
		except:
			messagebox.showwarning("FreedomWall","0.1 ~ 1 の間を設定してください。")
			return
		file_name = filedialog.askopenfilename(filetypes=[("動画ファイル","*.mp4"),("画像ファイル","*.png;*.jpg")])
		if not file_name:
			return

		if not GetWindow(title,"bubun"):
			messagebox.showwarning("FreedomWall",f"{title}があるタイトルのウィンドウが見つかりませんでした。")
			return

		self.data["windows"][title] = [file_name,alpha]
		with open("data.json","w") as f:
			dump(self.data,f,indent=4)
		self.window.reload()
		self.window.now = ""
		messagebox.showinfo("FreedomWall","設定しました。")

	# 壁紙の削除。
	def delete(self):
		title = simpledialog.askstring("FreedomWall","削除したい設定名を入力してください。")
		if not title:
			return
		if not title in self.data["windows"]:
			messagebox.showwarning("FreedomWall","その設定が見つかりませんでした。")
			return
		del self.data["windows"][title]
		with open("data.json","w") as f:
			dump(self.data,f,indent=4)
		self.window.reload()
		self.window.now = ""
		messagebox.showinfo("FreedomWall","その設定を削除しました。")

	# 壁紙一覧。
	def setting_list(self):
		desc = ", ".join(data for data in self.data["windows"].keys())
		messagebox.showinfo("FreedomWall",desc)

	# クレジット。
	def credit(self):
		messagebox.showinfo("FreedomWall",credit_text)

	# 終了。
	def exit(self):
		self.root.quit()
		self.window.video = None
		self.icon.shutdown()
		exit()

	# 実行。
	def run(self):
		self.icon.start()
