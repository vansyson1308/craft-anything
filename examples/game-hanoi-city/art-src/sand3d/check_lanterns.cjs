// Run with NODE_PATH pointing at an installation of gltf-validator.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const validator = require('gltf-validator');

async function main() {
  const hq = path.resolve(process.argv[2]);
  const rows = [];
  for (let i = 1; i <= 3; i++) {
    for (const lod of ['', '_lod1']) {
      const name = `den_long_0${i}${lod}.glb`;
      const data = fs.readFileSync(path.join(hq, 'street', name));
      const report = await validator.validateBytes(new Uint8Array(data), {uri: `street/${name}`});
      if (report.issues.numErrors || report.issues.numWarnings) throw new Error(`${name} failed glTF validation`);
      fs.writeFileSync(path.join(hq, 'reports', `${name}.validator.json`), JSON.stringify(report, null, 2) + '\n');
      rows.push({file: `street/${name}`, triangles: report.info.totalTriangleCount,
        materials: report.info.materialCount, draw_calls: report.info.drawCallCount,
        image_sizes: report.info.resources.filter(x => x.image).map(x => [x.image.width, x.image.height]),
        validator_errors: report.issues.numErrors, validator_warnings: report.issues.numWarnings,
        sha256: crypto.createHash('sha256').update(data).digest('hex')});
    }
  }
  fs.writeFileSync(path.join(hq, 'reports', 'lantern-checks.json'), JSON.stringify({scope: 'glTF Validator and measurements only; visual and in-game placement review remain pending', files: rows}, null, 2) + '\n');
  console.log(`Validated ${rows.length} lantern GLBs.`);
}
main().catch(e => {console.error(e); process.exitCode = 1;});
