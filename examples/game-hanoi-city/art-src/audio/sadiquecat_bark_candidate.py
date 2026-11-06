"""Prepare a review candidate from Sadiquecat's 850823 CC0 WAV.

No noise profile is invented: the short, pre-trimmed source has no reliable
separate recording of room tone. Requires FFmpeg, NumPy, SciPy, matplotlib.
"""
import argparse
import json
import subprocess
import tempfile
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly


def ffmpeg(*args):
    subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', *map(str, args)], check=True)


p = argparse.ArgumentParser()
p.add_argument('source', type=Path)
p.add_argument('output', type=Path)
args = p.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
sr, original = wavfile.read(args.source)
if sr != 48000 or original.ndim != 1 or original.dtype != np.int32:
    raise ValueError('Expected original 48 kHz, 24-bit mono WAV')
normalized = original.astype(np.float64) / 2**31
result = {'source_duration_s': len(original)/sr,
          'source_samples_above_0_999': int(np.count_nonzero(abs(normalized) >= .999)),
          'trim_start_s': 0.0, 'trim_end_s': .4,
          'notes': 'Original uploader describes a trimmed and normalized recording; no separate room-tone sample was available.'}
with tempfile.TemporaryDirectory() as td:
    td = Path(td)
    filtered = td / 'filtered.wav'
    ffmpeg('-i', args.source, '-af', 'atrim=end=0.4,highpass=f=80', '-c:a', 'pcm_f32le', filtered)
    rate, x = wavfile.read(filtered)
    x = x.astype(np.float64)
    fade_in, fade_out = round(.005*rate), round(.04*rate)
    x[:fade_in] *= np.linspace(0, 1, fade_in)
    x[-fade_out:] *= np.linspace(1, 0, fade_out)
    peak = max(np.max(np.abs(resample_poly(x, 4, 1))), 1e-12)
    gain = 10**(-1.2/20)/peak
    wav, decoded = td / 'input.wav', td / 'decoded.wav'
    ogg = args.output / 'cho_sua_04_candidate.ogg'
    # Refine after Vorbis encoding, which can move the peak.
    for _ in range(3):
        wavfile.write(wav, rate, (x*gain).astype(np.float32))
        ffmpeg('-i', wav, '-c:a', 'libvorbis', '-q:a', 6, ogg)
        ffmpeg('-i', ogg, '-c:a', 'pcm_f32le', decoded)
        _, y = wavfile.read(decoded)
        peak = max(np.max(np.abs(resample_poly(y, 4, 1))), 1e-12)
        db = 20*np.log10(peak)
        if abs(db + 1) < .03:
            break
        gain *= 10**((-1-db)/20)
    result.update(decoded_duration_s=len(y)/rate, true_peak_4x_dbfs=round(float(db), 3))
    fig, (a, b) = plt.subplots(2, 1, figsize=(9, 5), constrained_layout=True)
    a.plot(np.arange(len(y))/rate, y, linewidth=.5)
    a.set(xlabel='Seconds', ylabel='Amplitude', title=ogg.name)
    a.grid(True)
    b.specgram(y, Fs=rate, NFFT=512, noverlap=448, cmap='magma', vmin=-120, vmax=-30)
    b.set(xlabel='Seconds', ylabel='Frequency (Hz)', ylim=(0, 16000))
    fig.savefig(args.output / 'cho_sua_04_candidate.png', dpi=120)
    plt.close(fig)
(args.output / 'bark_04_measurements.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
print(json.dumps(result, indent=2))
