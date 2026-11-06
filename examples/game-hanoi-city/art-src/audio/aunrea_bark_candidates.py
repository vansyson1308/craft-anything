"""Create review-only dog bark candidates from aunrea's original WAV.

Requires FFmpeg, NumPy, SciPy and matplotlib. The input WAV is not distributed
here. Clipped source audio cannot be perfectly reconstructed; review by ear.
"""
import argparse
import json
import subprocess
import tempfile
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly


def run(*args):
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", *map(str, args)], check=True)


parser = argparse.ArgumentParser()
parser.add_argument("source", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
rate, source = wavfile.read(args.source)
if rate != 48000 or source.ndim != 1 or source.dtype != np.int32:
    raise ValueError("Expected the original 48 kHz, 24-bit mono WAV")
samples = source.astype(np.float64) / 2**31
stats = {"source_near_full_scale_samples": int(np.count_nonzero(np.abs(samples) >= .999)), "candidates": []}
segments = [(0.16, 0.66), (1.45, 1.96), (3.45, 3.94)]

with tempfile.TemporaryDirectory() as td:
    temp = Path(td)
    repaired = temp / "repaired.wav"
    run("-i", args.source, "-af", "adeclip", "-c:a", "pcm_f32le", repaired)
    for i, (start, end) in enumerate(segments, 1):
        cleaned = temp / f"clean-{i}.wav"
        # Feed quiet room tone first to learn the noise profile from this recording.
        graph = ("[0:a][1:a]concat=n=2:v=0:a=1,"
                 "asendcmd=c='0.0 afftdn sn start;0.3 afftdn sn stop',"
                 "afftdn=nr=9:nf=-60,atrim=start=0.4,asetpts=PTS-STARTPTS,highpass=f=80[a]")
        run("-ss", "2.65", "-t", "0.4", "-i", repaired,
            "-ss", start, "-t", end-start, "-i", repaired,
            "-filter_complex", graph, "-map", "[a]", "-c:a", "pcm_f32le", cleaned)
        sr, y = wavfile.read(cleaned)
        y = y.astype(np.float64)
        n_in, n_out = round(.005*sr), round(.04*sr)
        y[:n_in] *= np.linspace(0, 1, n_in)
        y[-n_out:] *= np.linspace(1, 0, n_out)
        true_peak = np.max(np.abs(resample_poly(y, 4, 1)))
        y *= 10**(-1.5/20) / max(true_peak, 1e-12)
        wav = temp / f"bark-{i}.wav"
        wavfile.write(wav, sr, y.astype(np.float32))
        ogg = args.output / f"cho_sua_{i:02d}_candidate.ogg"
        run("-i", wav, "-c:a", "libvorbis", "-q:a", "6", ogg)
        decoded = temp / f"decoded-{i}.wav"
        run("-i", ogg, "-c:a", "pcm_f32le", decoded)
        sr, x = wavfile.read(decoded)
        peak = 20 * np.log10(max(np.max(np.abs(resample_poly(x, 4, 1))), 1e-12))
        fig, (a, b) = plt.subplots(2, 1, figsize=(9, 5), constrained_layout=True)
        a.plot(np.arange(len(x))/sr, x, linewidth=.5)
        a.set(xlabel="Seconds", ylabel="Amplitude", title=ogg.name)
        a.grid(True)
        b.specgram(x, Fs=sr, NFFT=1024, noverlap=768, cmap="magma", vmin=-120, vmax=-30)
        b.set(xlabel="Seconds", ylabel="Frequency (Hz)", ylim=(0, 16000))
        fig.savefig(args.output / f"cho_sua_{i:02d}_candidate.png", dpi=120)
        plt.close(fig)
        stats["candidates"].append({"file": ogg.name, "source_start_s": start,
                                    "source_end_s": end, "decoded_duration_s": len(x)/sr,
                                    "true_peak_4x_dbfs": round(float(peak), 2)})

(args.output / "bark_measurements.json").write_text(json.dumps(stats, indent=2)+"\n", encoding="utf-8")
print(json.dumps(stats, indent=2))
