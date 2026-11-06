# Dog audio review candidates

These files are **drafts for listening and QC**, not approved game assets. They
do not replace the existing game sounds. The original recordings are CC0:

- [Dog Bark by aunrea](https://freesound.org/people/aunrea/sounds/495658/),
  48 kHz / 24-bit mono WAV, a medium-sized dog recorded with a Tascam
  DR-100MKIII and Audio-Technica AT875R.
- [Dog Whimper 1 by aunrea](https://freesound.org/people/aunrea/sounds/495656/),
  48 kHz / 24-bit mono WAV, recorded with the same equipment.
- [Dog bark 2 by Sadiquecat](https://freesound.org/people/Sadiquecat/sounds/850823/),
  48 kHz / 24-bit mono WAV, recorded with a DJI MIC-3. The uploader says this
  file has already been trimmed, normalized and faded.

The bark WAV contains three distinct barks, all clipped at full scale: 1,053
samples in the full source have absolute amplitude at least 0.999. FFmpeg
`adeclip` attempts reconstruction before denoising, but cannot restore lost
information reliably. The three Ogg excerpts and their frequency plots are
provided so a reviewer can judge the result; **a clean alternate recording is
still needed if clipping remains audible**. Four excerpts do not satisfy the
requirement for five distinct barking, growling and happy-dog variations.

`cho_sua_04_candidate.ogg` comes from the Sadiquecat recording. Only one source
sample exceeds 0.999 full scale, without a sustained flat-top clipping run.
The source is short and already faded, with no separate reliable room tone, so
no noise profile was fabricated. The candidate has an 80 Hz high-pass, trimmed
tail and short fades; its level was adjusted after Vorbis encoding.

The whimper is taken from 2.85–5.35 seconds of its source, avoiding a later
section with flattened peaks. Its 200 ms loop crossfade has a decoded seam
difference of about 0.00027 in normalized sample amplitude. A small numeric
seam alone does not establish that the loop sounds natural. An earlier
experiment from 7.35–9.55 seconds is superseded because the source peaks are
flattened around 0.5.

Measured on decoded Ogg with FFmpeg `astats` and `ebur128=peak=true`:

| Draft | Duration | Astats noise floor | True peak |
| --- | ---: | ---: | ---: |
| cho_sua_01 | 0.497 s | -50.8 dB | -2.1 dBFS |
| cho_sua_02 | 0.529 s | -49.0 dB | -1.8 dBFS |
| cho_sua_03 | 0.487 s | -52.9 dB | -1.7 dBFS |
| cho_sua_04 | 0.397 s | -50.6 dB | -1.0 dBFS |
| cho_rit | 2.3 s | -58.7 dB | -1.3 dBFS |

The second bark does not meet the requested -50 dB noise-floor threshold.
For very short clips the `astats` noise-floor estimate is sensitive to the
window and may include the decay of the sound; do not interpret it as a
separate silence recording. True peak is close to but not exactly -1 dBTP.
Short-term loudness, listener review, actual game playback and acceptance are
still pending. The cage-door sound is not addressed in this batch.

To reproduce, supply the original downloaded WAVs locally and run:

```text
python art-src/audio/aunrea_bark_candidates.py <dog-bark.wav> review/07/audio
python art-src/audio/sadiquecat_bark_candidate.py <sadiquecat-dog-bark-2.wav> review/07/audio
python art-src/audio/qc_whimper_candidate.py <dog-whimper-1.wav> <temporary-output-directory> --start 2.85 --duration 2.5 --noise-start 5.5
```

Both scripts require FFmpeg, NumPy, SciPy and matplotlib. The original source
recordings are not included in this draft.
