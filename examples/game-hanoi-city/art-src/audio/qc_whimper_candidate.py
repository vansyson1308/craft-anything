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
parser.add_argument('--start',type=float,default=1.0)
parser.add_argument('--duration',type=float,default=2.5)
parser.add_argument('--noise-start',type=float,default=9.0)
args=parser.parse_args()
source=args.source
out=args.output_dir
out.mkdir(exist_ok=True)
raw=out/'whimper_clean.wav'
# This tentative region may include more than one animal; listening review is required.
graph=("[0:a][1:a]concat=n=2:v=0:a=1,adeclick,"
       "asendcmd=c='0.0 afftdn sn start;0.35 afftdn sn stop',"
       "afftdn=nr=12:nf=-55,atrim=start=0.4,asetpts=PTS-STARTPTS,highpass=f=80[a]")
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y',
                '-ss',str(args.noise_start),'-t','0.4','-i',str(source),'-ss',str(args.start),'-t',str(args.duration),'-i',str(source),
                '-filter_complex',graph,'-map','[a]','-c:a','pcm_f32le',str(raw)],check=True)
rate,x=wavfile.read(raw)
n=round(rate*.2)
t=np.linspace(0,1,n,endpoint=False)
cross=x[-n:]*(1-t)+x[:n]*t
loop=np.concatenate((x[n:-n],cross))
# Rotate to a zero-crossing without changing adjacent samples at the seam.
crossings=np.where((loop[:-1]<=0)&(loop[1:]>0))[0]
idx=int(crossings[np.argmin(np.abs(loop[crossings]))]+1) if len(crossings) else 0
loop=np.roll(loop,-idx)
peak=np.max(np.abs(resample_poly(loop,4,1)))
loop*=10**(-1.3/20)/max(peak,1e-12)
wavfile.write(out/'whimper_loop.wav',rate,loop.astype(np.float32))
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(out/'whimper_loop.wav'),
                '-ar','44100','-ac','1','-c:a','libvorbis','-q:a','6',str(out/'cho_rit_candidate.ogg')],check=True)
decoded=out/'cho_rit_decoded.wav'
subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(out/'cho_rit_candidate.ogg'),
                '-c:a','pcm_f32le',str(decoded)],check=True)
rate,y=wavfile.read(decoded)
f=np.fft.rfftfreq(len(y),1/rate)
db=20*np.log10(np.maximum(np.abs(np.fft.rfft(y*np.hanning(len(y))))/(len(y)/2),1e-10))
fig,(a,b)=plt.subplots(2,1,figsize=(10,6),constrained_layout=True)
a.plot(np.arange(len(y))/rate,y,lw=.4);a.set_xlabel('Seconds');a.set_ylabel('Amplitude');a.grid(True)
b.plot(f,db,lw=.7);b.set_xlim(0,20000);b.set_ylim(-120,0);b.set_xlabel('Frequency (Hz)');b.set_ylabel('Magnitude (dBFS)');b.grid(True)
fig.savefig(out/'cho_rit_candidate.png',dpi=120)
print('decoded duration',len(y)/rate,'decoded loop jump',abs(float(y[0]-y[-1])))
