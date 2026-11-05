"""Convert selected CC0 source recordings into game-ready mono effects.

Sources: dog.7z by pauliuw and iron_door.ogg by themightyglider. See
assets/hq/SOURCES.md for source pages and licensing details.
"""
import argparse
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source_dir", type=Path)
    parser.add_argument("output_dir", type=Path)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    mappings = {
        "cho_sua_01": "dog-source/Dog/Dog Bark 1.wav",
        "cho_sua_02": "dog-source/Dog/Dog Bark 2.wav",
        "cho_sua_03": "dog-source/Dog/Dog Bark 3.wav",
        "cho_rit": "dog-source/Dog/Sad Dog 1.wav",
        "long_mo": "iron_door.ogg",
    }
    for name, source in mappings.items():
        # Short one-shot barks need a short silent tail for the loudness meter.
        # Tame the very short barks' transient peaks before loudness targeting;
        # retain the longer whine and gate opening without compression.
        if name.startswith("cho_sua"):
            treatment = ("acompressor=threshold=0.04:ratio=6:attack=0.5:release=50,"
                         "apad=pad_dur=0.5,")
        else:
            treatment = ""
        edge_fades = "afade=t=in:st=0:d=0.01,areverse,afade=t=in:st=0:d=0.025,areverse,"
        filters = f"{edge_fades}{treatment}loudnorm=I=-16:TP=-1:LRA=7"
        subprocess.run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
            "-i", str(args.source_dir / source), "-af", filters,
            "-ar", "44100", "-ac", "1", "-c:a", "libvorbis", "-q:a", "6",
            str(args.output_dir / f"{name}.ogg")
        ], check=True)


if __name__ == "__main__":
    main()
