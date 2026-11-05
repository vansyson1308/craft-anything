"""Supplementary audio-event labels; these are not a substitute for listening.

Uses a YAMNet ONNX model, its 521-class CSV, onnxruntime and FFmpeg. YAMNet's
0.96 s analysis window means short clips are zero-padded for analysis only.
"""
import argparse
import csv
import subprocess
import numpy as np
import onnxruntime as ort

parser=argparse.ArgumentParser()
parser.add_argument('model')
parser.add_argument('labels')
parser.add_argument('audio',nargs='+')
args=parser.parse_args()
with open(args.labels,encoding='utf-8',newline='') as f:
    labels=[row['display_name'] for row in csv.DictReader(f)]
session=ort.InferenceSession(args.model,providers=['CPUExecutionProvider'])
for path in args.audio:
    raw=subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-i',path,
                        '-ar','16000','-ac','1','-f','f32le','pipe:1'],check=True,stdout=subprocess.PIPE).stdout
    audio=np.frombuffer(raw,dtype='<f4').copy()
    if len(audio)<15600:audio=np.pad(audio,(0,15600-len(audio)))
    scores=session.run(None,{session.get_inputs()[0].name:audio})[0].mean(axis=0)
    indices=np.argsort(scores)[-6:][::-1]
    print(path,[(labels[i],round(float(scores[i]),3)) for i in indices])
