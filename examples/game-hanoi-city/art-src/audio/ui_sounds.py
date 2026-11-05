"""Create two original, synthesized Hanoi City UI cues in Ogg Vorbis.

Requires numpy and ffmpeg. No third-party samples or voices are used.
"""
import argparse
import subprocess
import tempfile
import wave
from pathlib import Path

import numpy as np

RATE = 44100


def envelope(t, attack, decay):
    return np.minimum(t / attack, 1.0) * np.exp(-t / decay) * (t >= 0)


def tap():
    t = np.arange(int(RATE * 0.60)) / RATE
    x = np.maximum(t - 0.025, 0)
    rng = np.random.default_rng(4180)
    tone = (np.sin(2 * np.pi * 690 * x) + .20 * np.sin(2 * np.pi * 1210 * x)) * envelope(x, .006, .14)
    noise = rng.standard_normal(len(t)) * envelope(x, .001, .012)
    return .23 * tone + .015 * noise


def hint():
    t = np.arange(int(RATE * 1.05)) / RATE
    out = np.zeros_like(t)
    for start, freq, weight in ((.035, 620, .7), (.165, 830, .58)):
        x = np.maximum(t - start, 0)
        env = envelope(x, .008, .20) * (t >= start)
        out += weight * env * (np.sin(2 * np.pi * freq * x) + .20 * np.sin(2 * np.pi * freq * 2.02 * x))
    return .22 * out


def write_wav(path, samples):
    pcm = np.clip(samples, -1, 1)
    with wave.open(str(path), "wb") as out:
        out.setnchannels(1)
        out.setsampwidth(2)
        out.setframerate(RATE)
        out.writeframes((pcm * 32767).astype("<i2").tobytes())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as directory:
        for name, make in (("ui_bam", tap), ("ui_meo", hint)):
            wav = Path(directory) / f"{name}.wav"
            write_wav(wav, make())
            subprocess.run([
                "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(wav),
                "-af", "loudnorm=I=-16:TP=-1:LRA=7", "-ar", str(RATE), "-ac", "1",
                "-c:a", "libvorbis", "-q:a", "6", str(args.output / f"{name}.ogg")
            ], check=True)


if __name__ == "__main__":
    main()
