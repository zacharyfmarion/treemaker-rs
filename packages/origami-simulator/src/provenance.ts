export const ORIGAMI_SIMULATOR_UPSTREAM = {
  repository: 'https://github.com/amandaghassaei/OrigamiSimulator',
  commit: '7855983a613c879c171b2b1557f8cd102d2640cf',
  license: 'MIT',
  solverFiles: [
    'js/dynamic/dynamicSolver.js',
    'js/model.js',
    'js/beam.js',
    'js/crease.js',
    'js/node.js',
    'index.html#normalCalc',
    'index.html#thetaCalcShader',
    'index.html#updateCreaseGeo',
    'index.html#velocityCalcShader',
    'index.html#positionCalcShader',
    'index.html#positionCalcVerletShader',
    'index.html#velocityCalcVerletShader',
  ],
} as const;
