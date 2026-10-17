"""Gera GIF otimizado e quadros de exibição diretamente do vídeo autorizado."""
import argparse
from pathlib import Path
import shutil
import subprocess
from PIL import Image

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("video", type=Path, help="Caminho do MP4 original fornecido pelo usuário")
args = parser.parse_args()
ffmpeg = shutil.which("ffmpeg")
if not ffmpeg:
    import imageio_ffmpeg
    ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
assets = Path(__file__).resolve().parent / "assets"
filters = "fps=12.5,scale=720:832:flags=lanczos"
base = [ffmpeg, "-hide_banner", "-loglevel", "error", "-i", str(args.video), "-t", "4.16"]
palette = filters + ",split[a][b];[a]palettegen=stats_mode=full[p];[b][p]paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle"
subprocess.run(base + ["-filter_complex", palette, "-frames:v", "52", "-an", "-loop", "-1", "-y", str(assets / "housinho.gif")], check=True)
raw = subprocess.run(base + ["-vf", filters, "-frames:v", "52", "-an", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"], capture_output=True, check=True).stdout
size = 720 * 832 * 3
if len(raw) != size * 52:
    raise ValueError("O vídeo deve fornecer 52 quadros para o trecho de 4,16 segundos.")
sprite = Image.new("RGB", (720 * 8, 832 * 7), (231, 231, 231))
for index in range(52):
    frame = Image.frombytes("RGB", (720, 832), raw[index * size:(index + 1) * size])
    sprite.paste(frame, ((index % 8) * 720, (index // 8) * 832))
sprite.save(assets / "housinho-frames.webp", quality=88, method=4)
print("GIF 720 × 832 e sprite gerados diretamente do MP4; 52 quadros / 4,16 s.")
