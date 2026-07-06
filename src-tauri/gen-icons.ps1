# Gera os ícones do WinButler: gravata-borboleta ciano sobre fundo escuro.
# Desenha via System.Drawing e escreve os PNG + um .ico multi-tamanho.
Add-Type -AssemblyName System.Drawing

$outDir = Join-Path $PSScriptRoot "icons"

# Cores do tema (dark + ciano).
$bgTop    = [System.Drawing.Color]::FromArgb(255, 15, 23, 42)   # slate-900
$bgBottom = [System.Drawing.Color]::FromArgb(255, 8, 13, 26)    # mais escuro
$cyan     = [System.Drawing.Color]::FromArgb(255, 34, 211, 238) # cyan-400
$cyanDark = [System.Drawing.Color]::FromArgb(255, 14, 165, 183) # nó da gravata

# Desenha o ícone num Bitmap do tamanho pedido e devolve o Bitmap.
function New-IconBitmap([int]$S) {
  $bmp = New-Object System.Drawing.Bitmap($S, $S, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.Clear([System.Drawing.Color]::Transparent)

  # Fundo: quadrado com cantos arredondados + degradê.
  $r = [int]($S * 0.22)
  $path = New-Object System.Drawing.Drawing2D.GraphicsPath
  $path.AddArc(0, 0, $r, $r, 180, 90)
  $path.AddArc($S - $r, 0, $r, $r, 270, 90)
  $path.AddArc($S - $r, $S - $r, $r, $r, 0, 90)
  $path.AddArc(0, $S - $r, $r, $r, 90, 90)
  $path.CloseFigure()
  $rect = New-Object System.Drawing.Rectangle(0, 0, $S, $S)
  $grad = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, $bgTop, $bgBottom, 90)
  $g.FillPath($grad, $path)

  # Borda ciano fina.
  $penW = [Math]::Max(1, [int]($S * 0.02))
  $pen = New-Object System.Drawing.Pen($cyan, $penW)
  $g.DrawPath($pen, $path)

  # Gravata-borboleta: dois triângulos que se encontram no centro + nó.
  $brush = New-Object System.Drawing.SolidBrush($cyan)
  function P([double]$x, [double]$y) { New-Object System.Drawing.PointF([single]($x * $S), [single]($y * $S)) }

  $left  = @((P 0.20 0.34), (P 0.20 0.66), (P 0.47 0.50))
  $right = @((P 0.80 0.34), (P 0.80 0.66), (P 0.53 0.50))
  $g.FillPolygon($brush, [System.Drawing.PointF[]]$left)
  $g.FillPolygon($brush, [System.Drawing.PointF[]]$right)

  # Nó central.
  $knotBrush = New-Object System.Drawing.SolidBrush($cyanDark)
  $kw = $S * 0.10; $kh = $S * 0.22
  $g.FillRectangle($knotBrush, [single]($S * 0.5 - $kw / 2), [single]($S * 0.5 - $kh / 2), [single]$kw, [single]$kh)

  $g.Dispose(); $grad.Dispose(); $pen.Dispose(); $brush.Dispose(); $knotBrush.Dispose(); $path.Dispose()
  return $bmp
}

# Salva um PNG do tamanho pedido.
function Save-Png([int]$S, [string]$name) {
  $bmp = New-IconBitmap $S
  $bmp.Save((Join-Path $outDir $name), [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
}

# PNGs que o tauri.conf referencia.
Save-Png 32  "32x32.png"
Save-Png 128 "128x128.png"
Save-Png 256 "128x128@2x.png"
Save-Png 512 "icon.png"

# .ico multi-tamanho (frames PNG, suportado no Windows Vista+).
$sizes = 16, 24, 32, 48, 64, 128, 256
$pngs = @()
foreach ($sz in $sizes) {
  $bmp = New-IconBitmap $sz
  $ms = New-Object System.IO.MemoryStream
  $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
  $pngs += , ($ms.ToArray())
  $bmp.Dispose(); $ms.Dispose()
}

$icoPath = Join-Path $outDir "icon.ico"
$fs = [System.IO.File]::Create($icoPath)
$bw = New-Object System.IO.BinaryWriter($fs)
$bw.Write([UInt16]0)                 # reservado
$bw.Write([UInt16]1)                 # tipo = ícone
$bw.Write([UInt16]$sizes.Count)      # qtd de imagens
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
  $sz = $sizes[$i]; $data = $pngs[$i]
  $b = if ($sz -ge 256) { 0 } else { $sz }
  $bw.Write([Byte]$b)                 # largura
  $bw.Write([Byte]$b)                 # altura
  $bw.Write([Byte]0)                  # cores da paleta
  $bw.Write([Byte]0)                  # reservado
  $bw.Write([UInt16]1)                # planos
  $bw.Write([UInt16]32)               # bits por pixel
  $bw.Write([UInt32]$data.Length)     # tamanho dos dados
  $bw.Write([UInt32]$offset)          # posição dos dados
  $offset += $data.Length
}
foreach ($data in $pngs) { $bw.Write($data) }
$bw.Flush(); $bw.Dispose(); $fs.Dispose()

Write-Host "Icones gerados em $outDir"
