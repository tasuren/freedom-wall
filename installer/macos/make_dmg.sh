if [ -e installer/macos/FreedomWall.dmg ]; then
  rm installer/macos/FreedomWall.dmg
fi
appdmg installer/macos/manifest.json installer/macos/FreedomWall.dmg