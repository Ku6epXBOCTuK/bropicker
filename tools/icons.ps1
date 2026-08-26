$ErrorActionPreference = "Stop"

$svg = "assets/icon.svg"

magick -background none $svg -resize 256x256 assets/icon.png
magick -background none $svg -define icon:auto-resize=256,128,64,48,32,24,16 assets/app.ico

foreach ($size in @(16, 24, 32, 44, 48, 50, 64, 88, 128, 150, 256, 512)) {
    magick -background none $svg -resize "${size}x${size}" ("assets/icon-{0}.png" -f $size)
}

Write-Host "icons regenerated: icon.png, app.ico, icon-{16..512}.png"
