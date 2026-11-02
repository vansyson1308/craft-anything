#!/usr/bin/env bash
# Downloads Hoàn Kiếm from the OpenStreetMap API, one 0.004° tile at a time (politely, one request
# a second), into the folder given (default osm/). Then build the city file with the importer:
#
#   cargo run --release --manifest-path ../game-hanoi-rescue/Cargo.toml --example osm_import -- \
#     assets/hoankiem.json 21.0288 105.8522 21.020 105.840 21.044 105.862 osm/t_*.osm
#
# Map data © OpenStreetMap contributors, under the Open Database Licence (ODbL).
set -u
OUT=${1:-osm}
mkdir -p "$OUT"
for lat in 21.016 21.020 21.024 21.028 21.032 21.036 21.040; do
  for lon in 105.838 105.842 105.846 105.850 105.854 105.858; do
    f="$OUT/t_${lat}_${lon}.osm"
    [ -s "$f" ] && continue
    lat2=$(awk "BEGIN { printf \"%.3f\", $lat + 0.004 }")
    lon2=$(awk "BEGIN { printf \"%.3f\", $lon + 0.004 }")
    curl -s -m 120 -o "$f" "https://api.openstreetmap.org/api/0.6/map?bbox=$lon,$lat,$lon2,$lat2"
    echo "$f $(stat -c %s "$f")"
    sleep 1
  done
done
