# Renders the app's authored orbit mark; Windows/System.Drawing only, no external artwork.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$assets = Join-Path (Split-Path $PSScriptRoot) 'app/assets'
$bitmap = [Drawing.Bitmap]::new(1024,1024)
$g = [Drawing.Graphics]::FromImage($bitmap)
$g.SmoothingMode = 'AntiAlias'
$g.ScaleTransform(16,16)
$shape = [Drawing.Drawing2D.GraphicsPath]::new()
$shape.AddArc(2,2,30,30,180,90); $shape.AddArc(32,2,30,30,270,90)
$shape.AddArc(32,32,30,30,0,90); $shape.AddArc(2,32,30,30,90,90); $shape.CloseFigure()
$brush = [Drawing.Drawing2D.LinearGradientBrush]::new([Drawing.Point]::new(0,0),[Drawing.Point]::new(0,64),[Drawing.ColorTranslator]::FromHtml('#29253b'),[Drawing.ColorTranslator]::FromHtml('#15151d'))
$rim = [Drawing.Pen]::new([Drawing.ColorTranslator]::FromHtml('#48415e'),1)
$orbit = [Drawing.Pen]::new([Drawing.ColorTranslator]::FromHtml('#ad9bff'),1.8)
$dot = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#d0c5ff'))
$g.FillPath($brush,$shape); $g.DrawPath($rim,$shape); $g.DrawEllipse($orbit,21,21,22,22)
$g.TranslateTransform(32,32); $g.RotateTransform(-35); $g.DrawEllipse($orbit,-24,-8,48,16)
$g.RotateTransform(35); $g.TranslateTransform(-32,-32); $g.FillEllipse($dot,48.7,15.7,4.6,4.6)
$g.Dispose(); $brush.Dispose(); $rim.Dispose(); $orbit.Dispose(); $dot.Dispose(); $shape.Dispose()
$frames = @()
foreach ($size in @(16,24,32,48,64,128,256)) {
    $frame = [Drawing.Bitmap]::new($size,$size)
    $graphics = [Drawing.Graphics]::FromImage($frame)
    $graphics.InterpolationMode = 'HighQualityBicubic'
    $graphics.DrawImage($bitmap,0,0,$size,$size); $graphics.Dispose()
    $stream = [IO.MemoryStream]::new(); $frame.Save($stream,[Drawing.Imaging.ImageFormat]::Png)
    if ($size -eq 256) { $frame.Save((Join-Path $assets 'planetes.png'),[Drawing.Imaging.ImageFormat]::Png) }
    $frames += ,@($size,$stream.ToArray()); $stream.Dispose(); $frame.Dispose()
}
$file = [IO.File]::Create((Join-Path $assets 'planetes.ico')); $writer = [IO.BinaryWriter]::new($file)
$writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]$frames.Count)
$offset = 6 + 16*$frames.Count
foreach ($frame in $frames) {
    $side = if ($frame[0] -eq 256) { 0 } else { $frame[0] }
    $writer.Write([byte]$side); $writer.Write([byte]$side); $writer.Write([uint16]0)
    $writer.Write([uint16]1); $writer.Write([uint16]32)
    $writer.Write([uint32]$frame[1].Length); $writer.Write([uint32]$offset); $offset += $frame[1].Length
}
foreach ($frame in $frames) { $writer.Write([byte[]]$frame[1]) }
$writer.Dispose(); $bitmap.Dispose()
