# OxiHuman Core Pack — Reconstruction Error

Honest quantisation error for the shipped **core** OHPK v1 pack. Each
morph target's sparse deltas are stored as `i16` max-abs quantised
values; the numbers below are recomputed by de-quantising the packed
pack and comparing against the original `.target` deltas.

* Pack: `assets/packs/oxihuman-core-v1.ohpk`
* Targets: 38
* Base vertices: 21833
* Model units: decimetres (1 unit = 100 mm)
* Base-mesh max position error: 0.012970 mm
* Base-mesh max UV error: 0.000008 (dimensionless)

**Worst-case target:** `macrodetails/height/female-young-averagemuscle-averageweight-maxheight` — max 0.0110 mm, RMS 0.0063 mm over 21819 affected vertices.

| Target | Affected verts | Max err (mm) | RMS err (mm) |
|---|---:|---:|---:|
| `macrodetails/height/female-young-averagemuscle-averageweight-maxheight` ⚠️ | 21819 | 0.0110 | 0.0063 |
| `macrodetails/height/male-young-averagemuscle-averageweight-maxheight` | 21819 | 0.0110 | 0.0063 |
| `macrodetails/height/female-young-averagemuscle-averageweight-minheight` | 21819 | 0.0056 | 0.0032 |
| `macrodetails/height/male-young-averagemuscle-averageweight-minheight` | 21819 | 0.0056 | 0.0032 |
| `macrodetails/asian-female-young` | 21819 | 0.0027 | 0.0015 |
| `macrodetails/universal-female-old-minmuscle-maxweight` | 10187 | 0.0019 | 0.0010 |
| `macrodetails/universal-male-young-minmuscle-maxweight` | 12065 | 0.0018 | 0.0010 |
| `macrodetails/universal-male-old-minmuscle-maxweight` | 12065 | 0.0017 | 0.0010 |
| `macrodetails/caucasian-female-young` | 21819 | 0.0014 | 0.0008 |
| `macrodetails/universal-female-young-minmuscle-maxweight` | 10484 | 0.0014 | 0.0007 |
| `macrodetails/universal-male-young-maxmuscle-maxweight` | 8833 | 0.0013 | 0.0007 |
| `macrodetails/african-female-young` | 21833 | 0.0013 | 0.0007 |
| `macrodetails/universal-female-old-maxmuscle-maxweight` | 8312 | 0.0011 | 0.0006 |
| `macrodetails/universal-female-young-maxmuscle-maxweight` | 8346 | 0.0011 | 0.0006 |
| `macrodetails/universal-male-old-minmuscle-minweight` | 9177 | 0.0011 | 0.0006 |
| `macrodetails/universal-male-old-maxmuscle-maxweight` | 8835 | 0.0010 | 0.0006 |
| `macrodetails/universal-male-young-minmuscle-minweight` | 9181 | 0.0010 | 0.0006 |
| `macrodetails/universal-female-young-minmuscle-minweight` | 21579 | 0.0010 | 0.0005 |
| `macrodetails/universal-female-old-minmuscle-minweight` | 13292 | 0.0008 | 0.0004 |
| `macrodetails/universal-male-old-maxmuscle-minweight` | 7601 | 0.0007 | 0.0004 |
| `measure/measure-bust-circ-incr` | 2061 | 0.0007 | 0.0004 |
| `macrodetails/universal-male-young-maxmuscle-minweight` | 7635 | 0.0006 | 0.0003 |
| `macrodetails/universal-female-old-maxmuscle-minweight` | 7415 | 0.0006 | 0.0003 |
| `measure/measure-hips-circ-incr` | 1482 | 0.0006 | 0.0003 |
| `measure/measure-underbust-circ-incr` | 2280 | 0.0005 | 0.0002 |
| `macrodetails/universal-female-old-averagemuscle-minweight` | 7449 | 0.0005 | 0.0003 |
| `macrodetails/universal-female-old-minmuscle-averageweight` | 5660 | 0.0005 | 0.0003 |
| `measure/measure-waist-circ-decr` | 1554 | 0.0005 | 0.0002 |
| `measure/measure-waist-circ-incr` | 1554 | 0.0005 | 0.0002 |
| `macrodetails/universal-female-young-maxmuscle-minweight` | 7482 | 0.0005 | 0.0003 |
| `measure/measure-bust-circ-decr` | 1964 | 0.0005 | 0.0002 |
| `macrodetails/universal-female-young-averagemuscle-minweight` | 7420 | 0.0004 | 0.0002 |
| `macrodetails/universal-female-old-maxmuscle-averageweight` | 7027 | 0.0004 | 0.0002 |
| `macrodetails/universal-female-young-maxmuscle-averageweight` | 6963 | 0.0004 | 0.0002 |
| `macrodetails/universal-female-young-averagemuscle-maxweight` | 6324 | 0.0003 | 0.0002 |
| `macrodetails/universal-female-old-averagemuscle-maxweight` | 7931 | 0.0003 | 0.0002 |
| `measure/measure-hips-circ-decr` | 1450 | 0.0003 | 0.0001 |
| `measure/measure-underbust-circ-decr` | 2229 | 0.0002 | 0.0001 |

## Reproduce

```sh
oxihuman pack-core --tier core --upstream assets/upstream/makehuman --out assets/packs/oxihuman-core-v1.ohpk --report docs/bench/pack-reconstruction-error.md
```
