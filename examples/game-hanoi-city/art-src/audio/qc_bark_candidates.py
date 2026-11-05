"""Produce unapproved QC candidates from the recordings cited in SOURCES.md."""
import argparse
from pathlib import Path
import subprocess
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

parser=argparse.ArgumentParser()
parser.add_argument('source_dir',type=Path,help='directory with source_2352.wav and source_2353.wav')
parser.add_argument('output_dir',type=Path,help='scratch output directory')
args=parser.parse_args()
root=args.source_dir
out=args.output_dir
out.mkdir(exist_ok=True)
regions=[('source_2352.wav',2.65,3.10,4.1),('source_2352.wav',3.70,4.05,4.1),
         ('source_2352.wav',4.90,5.20,4.1),('source_2353.wav',4.20,4.63,11.5),
         ('source_2353.wav',4.65,5.00,11.5)]
for index,(filename,start,end,noise_start) in enumerate(regions,1):
    source=root/filename
    temp=out/f'temp_{index}.wav'
    length=end-start
    graph=("[0:a][1:a]concat=n=2:v=0:a=1,adeclick,"
           "asendcmd=c='0.0 afftdn sn start;0.35 afftdn sn stop',"
           "afftdn=nr=12:nf=-55,atrim=start=0.4,asetpts=PTS-STARTPTS,"
           f"highpass=f=80,afade=t=in:st=0:d=0.005,afade=t=out:st={max(0,length-.04)}:d=0.04[a]")
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y',
                    '-ss',str(noise_start),'-t','0.4','-i',str(source),
                    '-ss',str(start),'-t',str(length),'-i',str(source),
                    '-filter_complex',graph,'-map','[a]','-c:a','pcm_f32le',str(temp)],check=True)
    rate,x=wavfile.read(temp)
    peak=np.max(np.abs(resample_poly(x,4,1)))
    gain=(10**(-1/20))/max(peak,1e-12)
    target=out/f'cho_sua_{index:02}.ogg'
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(temp),
                    '-af',f'volume={gain:.8f}','-ar','44100','-ac','1','-c:a','libvorbis','-q:a','6',str(target)],check=True)
    decoded=out/f'decoded_{index}.wav'
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(target),'-c:a','pcm_f32le',str(decoded)],check=True)
    rate,y=wavfile.read(decoded)
    truepeak=20*np.log10(np.max(np.abs(resample_poly(y,4,1))))
    # Vorbis can change inter-sample peaks; correct using the decoded result.
    gain*=10**((-1-truepeak)/20)
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(temp),
                    '-af',f'volume={gain:.8f}','-ar','44100','-ac','1','-c:a','libvorbis','-q:a','6',str(target)],check=True)
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(target),'-c:a','pcm_f32le',str(decoded)],check=True)
    rate,y=wavfile.read(decoded)
    truepeak=20*np.log10(np.max(np.abs(resample_poly(y,4,1))))
    f=np.fft.rfftfreq(len(y),1/rate)
    spec=20*np.log10(np.maximum(np.abs(np.fft.rfft(y*np.hanning(len(y))))/(len(y)/2),1e-10))
    fig,ax=plt.subplots(figsize=(10,4),constrained_layout=True)
    ax.plot(f,spec,lw=.7); ax.set_xlim(0,20000); ax.set_ylim(-120,0)
    ax.set_xlabel('Frequency (Hz)'); ax.set_ylabel('Magnitude (dBFS)'); ax.grid(True)
    ax.set_title(f'{target.name} | true peak {truepeak:.2f} dBTP | source window {start:.2f}-{end:.2f} s')
    fig.savefig(out/f'{target.stem}.png',dpi=120);plt.close(fig)
    print(target.name, 'duration',len(y)/rate,'truepeak',round(truepeak,2),'energy at 8-12k',round(float(np.max(spec[(f>=8000)&(f<=12000)])),1))
