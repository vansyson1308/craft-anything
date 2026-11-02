# Rebuilding the assets

`assets/` is committed, so nothing here is needed to build the game. To make it again from the
original sources:

1. **Map:** `tools/fetch_osm.sh osm`, then the `osm_import` command printed at the top of that
   script.
2. **Free packs** into one folder (`sources/`):
   - `base/`: Quaternius "Universal Base Characters [Standard]" (itch.io: quaternius.itch.io/universal-base-characters), unzipped
   - `anim/`: Quaternius "Universal Animation Library [Standard]" (quaternius.itch.io/universal-animation-library), unzipped
   - `acg/<Id>/`: ambientCG 1K-JPG zips, unzipped (`https://ambientcg.com/get?file=<Id>_1K-JPG.zip` for Asphalt031, PavingStones070, Plaster003, RoofingTiles006, Grass004, Bark012, Concrete034)
   - `Fox.glb`: `https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/Fox/glTF-Binary/Fox.glb`
3. `python3 -I tools/make_assets.py sources` (needs numpy and Pillow, and the DejaVu Sans Bold
   font at `/usr/share/fonts/truetype/dejavu/`).

The script paints clothes onto the characters (by where each triangle sits on the body), draws
the tube-house facades and their glow maps, recolours the dog, and prints the shop signs with
the real names of places near the old quarter.
