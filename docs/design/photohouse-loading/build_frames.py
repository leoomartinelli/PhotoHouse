"""Extrai os quadros do GIF autorizado para reprodução controlada no estudo."""
from pathlib import Path
from PIL import Image, ImageSequence

assets = Path(__file__).resolve().parent / "assets"
with Image.open(assets / "housinho.gif") as source:
    if source.size != (490, 566) or source.n_frames != 52:
        raise ValueError("Este estudo espera o GIF de 490 × 566 com 52 quadros.")
    sprite = Image.new("RGB", (490 * 8, 566 * 7), (231, 231, 231))
    for index, frame in enumerate(ImageSequence.Iterator(source)):
        if frame.info.get("duration") != 80:
            raise ValueError("Este estudo espera quadros de 80 ms.")
        sprite.paste(frame.convert("RGB"), ((index % 8) * 490, (index // 8) * 566))
sprite.save(assets / "housinho-frames.webp", quality=80, method=4)
print("52 quadros exportados em grade 8 × 7; o GIF original foi preservado.")
