if [ -e build/macos/FreedomWall.dmg ]; then
  rm build/macos/FreedomWall.dmg
fi
appdmg build/macos/manifest.json build/macos/FreedomWall.dmg