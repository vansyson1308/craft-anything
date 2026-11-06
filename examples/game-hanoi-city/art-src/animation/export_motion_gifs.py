"""Turn the even-frame review sequence into two looping 15 fps GIFs."""
import argparse
from pathlib import Path
from PIL import Image

p=argparse.ArgumentParser();p.add_argument('directory',type=Path);a=p.parse_args()
for name in ('wag','jump'):
    frames=[Image.open(a.directory/f'{name}_{i:02d}.png').convert('RGB') for i in range(0,30,2)]
    frames[0].save(a.directory/f'{name}.gif',save_all=True,append_images=frames[1:],duration=67,loop=0,optimize=True)
