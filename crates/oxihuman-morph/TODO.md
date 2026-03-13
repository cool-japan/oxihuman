# oxihuman-morph -- TODO

> Version: 0.1.1 | Updated: 2026-03-13

## Status: Stable

All core features implemented. 0 stubs. 5,865 passing tests (unit + doc).
918 source files, ~204k lines.

## Completed

### Core Engine
- [x] Parametric morphing engine (`HumanEngine`, `MeshBuffers`)
- [x] Morph target application and revert (`apply`, `diff`)
- [x] Parameter state management (`ParamState`)
- [x] Blendshape interpolation and weight curves
- [x] Delta cache and compression
- [x] Preset I/O and schema migration
- [x] Session history and undo
- [x] Shape comparison and search
- [x] Region-based morphing
- [x] Symmetry enforcement
- [x] Unit conversions and anthropometry
- [x] Fitting and measurements

### Animation and Retargeting
- [x] BVH parsing and retargeting (`anim_retarget`)
- [x] Pose interpolation (cubic Hermite, SQUAD, TCB tangents)
- [x] Motion clip warping, blending, looping, trimming
- [x] Animation layer system
- [x] MoCap retargeting with standard biped mapping
- [x] Expression calibration from facial landmarks

### FACS and Facial Expressions
- [x] Expression presets and component-based expressions
- [x] Expression physics (spring joints, impulse response)
- [x] Expression composer with layered blending
- [x] Facial rig with corrective shapes
- [x] Wrinkle map generation and deformation-driven wrinkles
- [x] Neural blend network for ML-driven morphing

### Lip Sync and Voice
- [x] Voice-driven animation (amplitude-to-jaw, viseme weights)
- [x] Advanced lip sync with phoneme events and coarticulation
- [x] Speech viseme system
- [x] Speech prosody emotion classification

### Pose and Skeleton
- [x] Pose retargeting with scale modes
- [x] Pose symmetry detection and enforcement
- [x] Blend shape graph evaluation (DAG-based)
- [x] FABRIK IK solver with cone constraints and pole vectors
- [x] XPBD secondary motion (jiggle/inertia chains)

### Body Shape Archetypes
- [x] Somatotypes: ectomorph, mesomorph, endomorph
- [x] Proportions: android, gynoid, hourglass, inverted-triangle, rectangle, pear, apple
- [x] Athletic build morph
- [x] BMI-driven body shape
- [x] Body composition (fat percentage, muscle definition)
- [x] Visceral and subcutaneous fat distribution
- [x] Body water/hydration effects

### Age Progression
- [x] Child morph (height/limb scaling)
- [x] Adolescent morph (sex-specific growth)
- [x] Elderly morph (height loss, kyphosis, skin sag)
- [x] Advanced age progression with aging curves

### Posture and Spinal
- [x] Posture morph (forward/lateral lean, sway)
- [x] Slouch, kyphosis, lordosis, scoliosis morphs
- [x] Spine curve morph (cervical/thoracic/lumbar)
- [x] Intervertebral disc morph
- [x] Sacrum, coccyx morphs

### Skeletal Morphs
- [x] Skull, mandible, orbital morphs
- [x] Femur, tibia/fibula, humerus, radius/ulna morphs
- [x] Carpals, tarsals morphs
- [x] Sternum, clavicle, scapula morphs
- [x] Rib cage morph
- [x] Iliac crest, pubic arch morphs

### Craniofacial Detail
- [x] Cranium height, occiput, parietal, occipital, frontal sinus morphs
- [x] Zygomatic arch/body, gonion, pogonion, gnathion morphs
- [x] Ramus, condyle, symphysis, coronoid morphs
- [x] Mastoid, supraorbital, infraorbital rim morphs
- [x] Glabella, nasolabial, marionette line morphs
- [x] Nasal: dorsum, root, septum, spine, columella, alar base, tip projection
- [x] Orbital: depth, rim, spacing
- [x] Temple: width, fossa
- [x] Malar: eminence, fat
- [x] Buccal fat, jowl, submental, temporal hollow morphs

### Facial Muscles
- [x] Frontalis, corrugator, orbicularis oculi morphs
- [x] Zygomaticus, depressor anguli, platysma morphs
- [x] Sternocleidomastoid, trapezius, masseter morphs
- [x] Muscle line deformation system

### Lip and Mouth Detail
- [x] Upper/lower lip body, roll, thickness morphs
- [x] Lip commissure, cupid's bow, vermillion border/width morphs
- [x] Philtrum: morph, depth, ridge
- [x] Oral commissure, labiomental, mentolabial morphs
- [x] Tooth shape control, tongue shape (v1/v2), uvula, soft palate

### Eye Detail
- [x] Pupil size control and dilation morph
- [x] Iris color blend with heterochromia
- [x] Sclera tone control
- [x] Eyelash density and curl
- [x] Eyebrow shape library
- [x] Eyelid crease, epicanthal fold, canthal tilt morphs
- [x] Lateral canthus, sclera show, lid fullness morphs
- [x] Brow bone bossing morph
- [x] Iris size morph

### Vocal Tract Anatomy
- [x] Pharynx, vocal tract, larynx position morphs
- [x] Thyroid/cricoid cartilage, arytenoid morphs
- [x] Glottis, trachea morphs
- [x] Diaphragm, rib cage expansion, abdomen expansion morphs
- [x] Pelvic floor morph

### Hair and Skin
- [x] Body hair system with region-based generation
- [x] Scalp hairline, hair part, volume, wave morphs
- [x] Beard density, mustache, sideburn morphs
- [x] Arm hair, eyebrow density, eyelash morphs
- [x] Skin color, texture scale, gloss, subsurface, translucency morphs
- [x] Skin pore morph, acne morph

### Limb and Extremity
- [x] Flat foot, high arch, knock knee, bow leg, pigeon toe morphs
- [x] Limb length asymmetry
- [x] Shoulder height, hip tilt, head tilt morphs
- [x] Facial/jaw asymmetry morphs
- [x] Hand v2 (dorsum, knuckle, tendon, vein), palm, thumb controls
- [x] Ankle shape morph, knee shape morph

### Fine Controls (60+ parametric controllers)
- [x] Eye inner/outer corner controls
- [x] Face vertical/contour controls
- [x] Forehead raise/tension, brow wrinkle/furrowing controls
- [x] Cheek puff depth, cheek rise controls
- [x] Chin recess/recession controls
- [x] Jaw shift/twist controls
- [x] Lip purse control
- [x] Nasal root/flare/spine controls
- [x] Neck tilt/flexion/crease controls
- [x] Shoulder pad/slope controls
- [x] Scapula, shin, thigh controls
- [x] Body lean/twist controls
- [x] Body volume control
- [x] Ear rim/helix fold controls
- [x] Foot ball/toe spread controls
- [x] Hand grip/vein controls

### Tools and Utilities
- [x] Morph quantization and compression
- [x] Delta painter (brush-based morph editing)
- [x] Target tools (add/subtract/mirror/scale/merge/validate targets)
- [x] Breathing simulation
- [x] Body language and emotion mapping
- [x] Crowd variation generator

## Future Work

(No outstanding TODO/FIXME markers found in source.)
