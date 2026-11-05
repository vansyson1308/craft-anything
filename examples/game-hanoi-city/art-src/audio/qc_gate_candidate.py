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
parser.add_argument('source',type=Path)
parser.add_argument('output_dir',type=Path)
args=parser.parse_args()
source=args.source
out=args.output_dir
out.mkdir(exist_ok=True)
temp=out/'gate_clean.wav'
graph=("[0:a][1:a]concat=n=2:v=0:a=1,adeclick,"
       "asendcmd=c='0.0 afftdn sn start;0.35 afftdn sn stop',afftdn=nr=12:nf=-55,"
       "atrim=start=0.4,asetpts=PTS-STARTPTS,highpass=f=80,"
       "afade=t=in:st=0:d=0.005,afade=t=out:st=0.56:d=0.04[a]")
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y',
                '-ss','4.0','-t','0.4','-i',str(source),'-ss','0.45','-t','0.6','-i',str(source),
                '-filter_complex',graph,'-map','[a]','-c:a','pcm_f32le',str(temp)],check=True)
rate,x=wavfile.read(temp)
gain=10**(-1.2/20)/max(np.max(np.abs(resample_poly(x,4,1))),1e-12)
target=out/'long_mo_candidate.ogg'
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(temp),'-af',f'volume={gain}',
                '-ar','44100','-ac','1','-c:a','libvorbis','-q:a','6',str(target)],check=True)
decoded=out/'long_mo_decoded.wav'
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(target),'-c:a','pcm_f32le',str(decoded)],check=True)
rate,y=wavfile.read(decoded)
truepeak=20*np.log10(np.max(np.abs(resample_poly(y,4,1))))
f=np.fft.rfftfreq(len(y),1/rate)
db=20*np.log10(np.maximum(np.abs(np.fft.rfft(y*np.hanning(len(y))))/(len(y)/2),1e-10))
fig,ax=plt.subplots(figsize=(10,4),constrained_layout=True)
ax.plot(f,db,lw=.7);ax.set_xlim(0,20000);ax.set_ylim(-120,0);ax.set_xlabel('Frequency (Hz)');ax.set_ylabel('Magnitude (dBFS)');ax.grid(True)
fig.savefig(out/'long_mo_candidate.png',dpi=120)
print('duration',len(y)/rate,'truepeak',round(truepeak,2),'highest bin above -90',round(float(f[db>=-90].max())))
