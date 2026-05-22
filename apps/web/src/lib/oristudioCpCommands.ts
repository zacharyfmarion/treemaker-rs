export const ORISTUDIO_CP_COMMAND_GROUPS = [
  {
    id: 'select-edit',
    label: 'Select and edit',
    railLabel: 'Select',
    order: 10,
  },
  {
    id: 'draw',
    label: 'Draw folds and points',
    railLabel: 'Draw',
    order: 20,
  },
  {
    id: 'construct',
    label: 'Construct by geometry',
    railLabel: 'Construct',
    order: 30,
  },
  {
    id: 'transform',
    label: 'Transform and operation frame',
    railLabel: 'Transform',
    order: 40,
  },
  {
    id: 'color',
    label: 'Color and assignment',
    railLabel: 'Color',
    order: 50,
  },
  {
    id: 'annotations',
    label: 'Circles, text, and annotations',
    railLabel: 'Annotate',
    order: 60,
  },
  {
    id: 'generators',
    label: 'Generators and base molecules',
    railLabel: 'Generate',
    order: 70,
  },
  {
    id: 'measure',
    label: 'Measure',
    railLabel: 'Measure',
    order: 80,
  },
  {
    id: 'check-fix',
    label: 'Check and fix',
    railLabel: 'Check',
    order: 90,
  },
  {
    id: 'folding',
    label: 'Fold estimate and folded figure',
    railLabel: 'Fold',
    order: 100,
  },
  {
    id: 'file',
    label: 'File import/export',
    railLabel: 'File',
    order: 110,
  },
] as const;

export type OristudioCpCommandGroupId = (typeof ORISTUDIO_CP_COMMAND_GROUPS)[number]['id'];

export type OristudioCpCommandPlacement =
  | 'left-rail'
  | 'left-rail-overflow'
  | 'menu'
  | 'palette'
  | 'hidden-ui-only';

export type OristudioCpCommandUiStatus =
  | 'not-implemented'
  | 'porting'
  | 'ready'
  | 'out-of-scope-ui';

export type OristudioCpOperationStatus =
  | 'Unsupported'
  | 'Porting'
  | 'UnitTested'
  | 'OracleTested'
  | 'DocumentedDifference'
  | 'OutOfScopeUi';

export interface OristudioCpCommandDefinition {
  id: `cp.${string}`;
  operationId: OristudioCpOperationId;
  label: string;
  group: OristudioCpCommandGroupId;
  placement: OristudioCpCommandPlacement;
  icon: string;
  upstream: string;
  tooltip: string;
  uiStatus: OristudioCpCommandUiStatus;
  disabledReason: string;
  selectionRequirement?: string;
  shortcut?: string;
  toolSteps?: readonly string[];
}

function notImplemented(
  operationId: OristudioCpOperationId,
  label: string,
  group: OristudioCpCommandGroupId,
  icon: string,
  upstream: string,
  options: Partial<
    Pick<
      OristudioCpCommandDefinition,
      'placement' | 'selectionRequirement' | 'shortcut' | 'toolSteps' | 'tooltip'
    >
  > = {}
): OristudioCpCommandDefinition {
  return {
    id: commandId(operationId),
    operationId,
    label,
    group,
    placement: options.placement ?? 'left-rail',
    icon,
    upstream,
    tooltip: options.tooltip ?? label,
    uiStatus: 'not-implemented',
    disabledReason: 'Not implemented in the CP editor yet',
    selectionRequirement: options.selectionRequirement,
    shortcut: options.shortcut,
    toolSteps: options.toolSteps,
  };
}

function porting(
  operationId: OristudioCpOperationId,
  label: string,
  group: OristudioCpCommandGroupId,
  icon: string,
  upstream: string,
  options: Partial<
    Pick<
      OristudioCpCommandDefinition,
      'placement' | 'selectionRequirement' | 'shortcut' | 'toolSteps' | 'tooltip'
    >
  > = {}
): OristudioCpCommandDefinition {
  return {
    ...notImplemented(operationId, label, group, icon, upstream, options),
    uiStatus: 'porting',
    disabledReason: 'Kernel port is in progress; UI wiring is not implemented yet',
  };
}

function ready(
  operationId: OristudioCpOperationId,
  label: string,
  group: OristudioCpCommandGroupId,
  icon: string,
  upstream: string,
  options: Partial<
    Pick<
      OristudioCpCommandDefinition,
      'placement' | 'selectionRequirement' | 'shortcut' | 'toolSteps' | 'tooltip'
    >
  > = {}
): OristudioCpCommandDefinition {
  return {
    ...notImplemented(operationId, label, group, icon, upstream, options),
    uiStatus: 'ready',
    disabledReason: 'Ready',
  };
}

function outOfScopeUi(
  operationId: OristudioCpOperationId,
  label: string,
  group: OristudioCpCommandGroupId,
  icon: string,
  upstream: string,
  tooltip: string
): OristudioCpCommandDefinition {
  return {
    id: commandId(operationId),
    operationId,
    label,
    group,
    placement: 'hidden-ui-only',
    icon,
    upstream,
    tooltip,
    uiStatus: 'out-of-scope-ui',
    disabledReason: 'Handled by the viewport runtime instead of a CP command',
  };
}

function commandId(operationId: string): `cp.${string}` {
  return `cp.${operationId.replace(/[A-Z]/g, (letter, index) => `${index ? '-' : ''}${letter.toLowerCase()}`)}`;
}

export const ORISTUDIO_CP_COMMANDS: OristudioCpCommandDefinition[] = [
  notImplemented('DrawCreaseFree', 'Draw crease', 'draw', 'pen-line', 'MouseHandlerDrawCreaseFree', {
    shortcut: 'L',
    toolSteps: ['Pick start point', 'Pick end point'],
  }),
  outOfScopeUi(
    'MoveCreasePattern',
    'Pan viewport',
    'select-edit',
    'hand',
    'MouseHandlerMoveCreasePattern',
    'Covered by the landed CP viewport pan controls'
  ),
  ready(
    'LineSegmentDelete',
    'Delete line segment',
    'select-edit',
    'eraser',
    'MouseHandlerLineSegmentDelete',
    {
      selectionRequirement: 'selected line segment',
      tooltip: 'Delete selected line segments',
    }
  ),
  notImplemented('ChangeCreaseType', 'Change crease type', 'color', 'paintbrush', 'MouseHandlerChangeCreaseType'),
  notImplemented('LengthenCrease', 'Lengthen crease', 'transform', 'stretch-horizontal', 'MouseHandlerLengthenCrease'),
  notImplemented('SquareBisector', 'Square bisector', 'construct', 'square-dashed', 'MouseHandlerSquareBisector'),
  notImplemented('Inward', 'Inward fold line', 'construct', 'corner-down-left', 'MouseHandlerInward'),
  notImplemented('PerpendicularDraw', 'Perpendicular draw', 'construct', 'ruler', 'MouseHandlerPerpendicularDraw'),
  notImplemented('SymmetricDraw', 'Symmetric draw', 'construct', 'flip-horizontal', 'MouseHandlerSymmetricDraw'),
  notImplemented('DrawCreaseRestricted', 'Draw restricted crease', 'draw', 'pen-tool', 'MouseHandlerDrawCreaseRestricted'),
  notImplemented('DrawCreaseSymmetric', 'Mirror selected creases', 'construct', 'copy-plus', 'MouseHandlerDrawCreaseSymmetric'),
  notImplemented('DrawCreaseAngleRestricted', 'Angle restricted crease', 'construct', 'drafting-compass', 'MouseHandlerDrawCreaseAngleRestricted'),
  notImplemented('DrawPoint', 'Draw point', 'draw', 'circle-dot', 'MouseHandlerDrawPoint', {
    toolSteps: ['Pick point'],
  }),
  notImplemented('DeletePoint', 'Delete point', 'select-edit', 'circle-x', 'MouseHandlerDeletePoint'),
  notImplemented('AngleSystem', 'Angle system', 'construct', 'chart-no-axes-combined', 'MouseHandlerAngleSystem'),
  notImplemented('DrawCreaseAngleRestricted3', 'Angle restricted 3 crease', 'construct', 'between-horizontal-start', 'MouseHandlerDrawCreaseAngleRestricted3_2'),
  notImplemented('CreaseSelect', 'Select crease', 'select-edit', 'mouse-pointer-2', 'MouseHandlerCreaseSelect', {
    shortcut: 'V',
  }),
  notImplemented('CreaseUnselect', 'Unselect crease', 'select-edit', 'mouse-pointer-click', 'MouseHandlerCreaseUnselect'),
  ready('CreaseMove', 'Move selected creases', 'transform', 'move', 'MouseHandlerCreaseMove', {
    selectionRequirement: 'selected creases',
    toolSteps: ['Pick source point', 'Pick destination point'],
  }),
  ready('CreaseCopy', 'Copy selected creases', 'transform', 'copy', 'MouseHandlerCreaseCopy', {
    selectionRequirement: 'selected creases',
    toolSteps: ['Pick source point', 'Pick destination point'],
  }),
  ready('CreaseMakeMountain', 'Make mountain', 'color', 'mountain', 'MouseHandlerCreaseMakeMountain', {
    selectionRequirement: 'selected lines',
    tooltip: 'Make selected lines mountain folds',
  }),
  ready('CreaseMakeValley', 'Make valley', 'color', 'waves', 'MouseHandlerCreaseMakeValley', {
    selectionRequirement: 'selected lines',
    tooltip: 'Make selected lines valley folds',
  }),
  ready('CreaseMakeEdge', 'Make edge', 'color', 'box-select', 'MouseHandlerCreaseMakeEdge', {
    selectionRequirement: 'selected lines',
    tooltip: 'Make selected lines edge folds',
  }),
  outOfScopeUi(
    'BackgroundChangePosition',
    'Move background',
    'select-edit',
    'image',
    'MouseHandlerBackgroundChangePosition',
    'Background image manipulation is UI-only and not part of the CP kernel'
  ),
  notImplemented('LineSegmentDivision', 'Divide line by count', 'draw', 'split', 'MouseHandlerLineSegmentDivision'),
  notImplemented('LineSegmentRatioSet', 'Divide line by ratio', 'draw', 'divide', 'MouseHandlerLineSegmentRatioSet'),
  notImplemented('PolygonSetNoCorners', 'Regular polygon', 'generators', 'hexagon', 'MouseHandlerPolygonSetNoCorners'),
  notImplemented('CreaseAdvanceType', 'Advance crease type', 'color', 'list-restart', 'MouseHandlerCreaseAdvanceType'),
  ready('CreaseMove4p', 'Move by four points', 'transform', 'scan-line', 'MouseHandlerCreaseMove4p', {
    selectionRequirement: 'selected creases',
    toolSteps: [
      'Pick source first point',
      'Pick source second point',
      'Pick target first point',
      'Pick target second point',
    ],
  }),
  ready('CreaseCopy4p', 'Copy by four points', 'transform', 'scan-line', 'MouseHandlerCreaseCopy4p', {
    selectionRequirement: 'selected creases',
    toolSteps: [
      'Pick source first point',
      'Pick source second point',
      'Pick target first point',
      'Pick target second point',
    ],
  }),
  notImplemented('FishBoneDraw', 'Fishbone draw', 'construct', 'git-branch', 'MouseHandlerFishBoneDraw'),
  notImplemented('CreaseMakeMv', 'Make alternating M/V', 'color', 'git-branch', 'MouseHandlerCreaseMakeMV'),
  notImplemented('DoubleSymmetricDraw', 'Double symmetric draw', 'construct', 'fold-horizontal', 'MouseHandlerDoubleSymmetricDraw'),
  notImplemented('CreasesAlternateMv', 'Alternate crossing M/V', 'color', 'shuffle', 'MouseHandlerCreasesAlternateMV'),
  notImplemented('DrawCreaseAngleRestricted5', 'Angle restricted 5 crease', 'construct', 'chart-pie', 'MouseHandlerDrawCreaseAngleRestricted5'),
  notImplemented('VertexMakeAngularlyFlatFoldable', 'Make vertex flat-foldable', 'construct', 'badge-check', 'MouseHandlerVertexMakeAngularlyFlatFoldable'),
  notImplemented('FoldableLineInput', 'Foldable line input', 'construct', 'list-plus', 'MouseHandlerFoldableLineInput'),
  notImplemented('ParallelDraw', 'Parallel draw', 'construct', 'align-justify', 'MouseHandlerParallelDraw'),
  notImplemented('VertexDeleteOnCrease', 'Delete vertex on crease', 'select-edit', 'scan-x', 'MouseHandlerVertexDeleteOnCrease'),
  notImplemented('CircleDraw', 'Draw circle', 'annotations', 'circle', 'MouseHandlerCircleDraw'),
  notImplemented('CircleDrawThreePoint', 'Circle through three points', 'annotations', 'circle-dot', 'MouseHandlerCircleDrawThreePoint'),
  notImplemented('CircleDrawSeparate', 'Separate circle', 'annotations', 'circle-dashed', 'MouseHandlerCircleDrawSeparate'),
  notImplemented('CircleDrawTangentLine', 'Circle tangent line', 'annotations', 'circle-slash', 'MouseHandlerCircleDrawTangentLine'),
  notImplemented('CircleDrawInverted', 'Inverted circle', 'annotations', 'refresh-cw', 'MouseHandlerCircleDrawInverted'),
  notImplemented('CircleDrawFree', 'Free circle', 'annotations', 'circle-plus', 'MouseHandlerCircleDrawFree'),
  notImplemented('CircleDrawConcentric', 'Concentric circle', 'annotations', 'circle-dot-dashed', 'MouseHandlerCircleDrawConcentric'),
  notImplemented('CircleDrawConcentricSelect', 'Concentric from selection', 'annotations', 'circle-dot', 'MouseHandlerCircleDrawConcentricSelect'),
  notImplemented('CircleDrawConcentricTwoCircleSelect', 'Concentric from two circles', 'annotations', 'venetian-mask', 'MouseHandlerCircleDrawConcentricTwoCircleSelect'),
  notImplemented('ParallelDrawWidth', 'Parallel draw by width', 'construct', 'between-horizontal-end', 'MouseHandlerParallelDrawWidth'),
  notImplemented('ContinuousSymmetricDraw', 'Continuous symmetric draw', 'construct', 'repeat', 'MouseHandlerContinuousSymmetricDraw'),
  notImplemented('DisplayLengthBetweenPoints1', 'Measure length 1', 'measure', 'ruler', 'MouseHandlerDisplayLengthBetweenPoints', {
    toolSteps: ['Pick first point', 'Pick second point'],
  }),
  notImplemented('DisplayLengthBetweenPoints2', 'Measure length 2', 'measure', 'ruler-dimension-line', 'MouseHandlerDisplayLengthBetweenPoints', {
    toolSteps: ['Pick first point', 'Pick second point'],
  }),
  notImplemented('DisplayAngleBetweenThreePoints1', 'Measure angle 1', 'measure', 'angle', 'MouseHandlerDisplayAngleBetweenThreePoints', {
    toolSteps: ['Pick first point', 'Pick vertex point', 'Pick second point'],
  }),
  notImplemented('DisplayAngleBetweenThreePoints2', 'Measure angle 2', 'measure', 'angle', 'MouseHandlerDisplayAngleBetweenThreePoints', {
    toolSteps: ['Pick first point', 'Pick vertex point', 'Pick second point'],
  }),
  notImplemented('DisplayAngleBetweenThreePoints3', 'Measure angle 3', 'measure', 'angle', 'MouseHandlerDisplayAngleBetweenThreePoints', {
    toolSteps: ['Pick first point', 'Pick vertex point', 'Pick second point'],
  }),
  ready('CreaseToggleMv', 'Toggle mountain/valley', 'color', 'repeat-2', 'MouseHandlerCreaseToggleMV', {
    selectionRequirement: 'selected mountain or valley lines',
    tooltip: 'Toggle selected mountain and valley lines',
  }),
  notImplemented('CircleChangeColor', 'Change circle color', 'annotations', 'palette', 'MouseHandlerCircleChangeColor'),
  ready('CreaseMakeAux', 'Make auxiliary', 'color', 'scan-line', 'MouseHandlerCreaseMakeAux', {
    selectionRequirement: 'selected folding lines',
    tooltip: 'Convert selected folding lines to auxiliary lines',
  }),
  notImplemented('OperationFrameCreate', 'Operation frame', 'transform', 'frame', 'MouseHandlerOperationFrameCreate', {
    toolSteps: ['Drag operation frame'],
  }),
  notImplemented('VoronoiCreate', 'Voronoi', 'generators', 'network', 'MouseHandlerVoronoiCreate'),
  notImplemented('FlatFoldableCheck', 'Flat-foldable boundary check', 'check-fix', 'shield-check', 'MouseHandlerFlatFoldableCheck'),
  ready('CreaseDeleteOverlapping', 'Delete overlapping creases', 'select-edit', 'combine', 'MouseHandlerCreaseDeleteOverlapping', {
    toolSteps: ['Pick drag start point', 'Pick drag end point'],
    tooltip: 'Delete crease segments overlapping a dragged line',
  }),
  ready('CreaseDeleteIntersecting', 'Delete intersecting creases', 'select-edit', 'unlink', 'MouseHandlerCreaseDeleteIntersecting', {
    toolSteps: ['Pick drag start point', 'Pick drag end point'],
    tooltip: 'Delete crease segments intersecting or overlapping a dragged line',
  }),
  notImplemented('SelectPolygon', 'Select polygon', 'select-edit', 'lasso-select', 'MouseHandlerSelectPolygon'),
  notImplemented('UnselectPolygon', 'Unselect polygon', 'select-edit', 'lasso', 'MouseHandlerUnselectPolygon'),
  ready('SelectLineIntersecting', 'Select intersecting line', 'select-edit', 'scan-search', 'MouseHandlerSelectLineIntersecting', {
    toolSteps: ['Pick drag start point', 'Pick drag end point'],
    tooltip: 'Select crease segments intersecting or overlapping a dragged line',
  }),
  ready('UnselectLineIntersecting', 'Unselect intersecting line', 'select-edit', 'scan-search', 'MouseHandlerUnselectLineIntersecting', {
    toolSteps: ['Pick drag start point', 'Pick drag end point'],
    tooltip: 'Unselect crease segments intersecting or overlapping a dragged line',
  }),
  notImplemented('LengthenCreaseSameColor', 'Lengthen same color', 'transform', 'stretch-horizontal', 'MouseHandlerLengthenCreaseSameColor'),
  notImplemented('FoldableLineDraw', 'Foldable line draw', 'construct', 'pen-line', 'MouseHandlerFoldableLineDraw'),
  notImplemented('ReplaceLineTypeSelect', 'Replace selected line type', 'color', 'replace', 'MouseHandlerReplaceTypeSelect'),
  notImplemented('DeleteLineTypeSelect', 'Delete selected line type', 'color', 'trash-2', 'MouseHandlerDeleteTypeSelect'),
  notImplemented('SelectLasso', 'Select lasso', 'select-edit', 'lasso-select', 'MouseHandlerSelectLasso'),
  notImplemented('UnselectLasso', 'Unselect lasso', 'select-edit', 'lasso', 'MouseHandlerUnselectLasso'),
  notImplemented('Text', 'Text annotation', 'annotations', 'text-cursor-input', 'MouseHandlerText'),
  notImplemented('DrawBlintz', 'Blintz base', 'generators', 'sparkles', 'MouseHandlerDrawBlintz'),
  notImplemented('DrawFishBase', 'Fish base', 'generators', 'sparkles', 'MouseHandlerDrawFishBase'),
  notImplemented('DrawDoveBase', 'Dove base', 'generators', 'sparkles', 'MouseHandlerDrawDoveBase'),
  notImplemented('DrawBirdBase', 'Bird base', 'generators', 'sparkles', 'MouseHandlerDrawBirdBase'),
  notImplemented('DrawFrogBase', 'Frog base', 'generators', 'sparkles', 'MouseHandlerDrawFrogBase'),
  notImplemented('ModifyCalculatedShape', 'Modify calculated shape', 'folding', 'pen-tool', 'MouseHandlerModifyCalculatedShape'),
  notImplemented('MoveCalculatedShape', 'Move calculated shape', 'folding', 'move', 'MouseHandlerMoveCalculatedShape'),
  notImplemented('ChangeStandardFace', 'Change standard face', 'folding', 'layers', 'MouseHandlerChangeStandardFace'),
  notImplemented('AddFoldingConstraint', 'Add folding constraint', 'folding', 'list-plus', 'MouseHandlerAddFoldingConstraints'),
  notImplemented('Axiom5', 'Axiom 5', 'construct', 'compass', 'MouseHandlerAxiom5'),
  notImplemented('Axiom7', 'Axiom 7', 'construct', 'compass', 'MouseHandlerAxiom7'),
  ready('FixInaccurate', 'Fix inaccurate creases', 'check-fix', 'wrench', 'MouseHandlerCreaseFixInaccurate', {
    selectionRequirement: 'selected folding lines',
    tooltip: 'Snap inaccurate selected folding lines to Oriedita fix targets',
  }),
  notImplemented('ImportCp', 'Import CP', 'file', 'file-input', 'CpImporter', { placement: 'menu' }),
  notImplemented('ExportCp', 'Export CP', 'file', 'file-output', 'CpExporter', { placement: 'menu' }),
  notImplemented('ImportFold', 'Import FOLD', 'file', 'file-input', 'FoldImporter', { placement: 'menu' }),
  notImplemented('ExportFold', 'Export FOLD', 'file', 'file-output', 'FoldExporter', { placement: 'menu' }),
  notImplemented('ImportOri', 'Import ORI', 'file', 'file-input', 'OriImporter', { placement: 'menu' }),
  notImplemented('ExportOri', 'Export ORI', 'file', 'file-output', 'OriExporter', { placement: 'menu' }),
  notImplemented('ImportOrh', 'Import ORH', 'file', 'file-input', 'OrhImporter', { placement: 'menu' }),
  notImplemented('ExportOrh', 'Export ORH', 'file', 'file-output', 'OrhExporter', { placement: 'menu' }),
  notImplemented('ImportObj', 'Import OBJ', 'file', 'file-input', 'ObjImporter', { placement: 'menu' }),
  notImplemented('ExportDxf', 'Export DXF', 'file', 'file-output', 'DxfExporter', { placement: 'menu' }),
  notImplemented('SaveConvert', 'Convert save', 'file', 'file-cog', 'SaveConverter', { placement: 'palette' }),
  notImplemented('SaveVersionDetect', 'Detect save version', 'file', 'file-search', 'FileVersionTester', {
    placement: 'palette',
  }),
  notImplemented('CheckCamv', 'Check CAMV', 'check-fix', 'shield-alert', 'CheckCAMVTask'),
  porting('FoldingEstimate', 'Fold estimate', 'folding', 'origami', 'FoldingEstimateTask'),
  porting('FoldingEstimateSpecific', 'Fold to case', 'folding', 'list-ordered', 'FoldingEstimateSpecificTask'),
  porting('FoldingEstimateSave100', 'Save 100 simulations', 'folding', 'download', 'FoldingEstimateSave100Task'),
  porting('TwoColoredCp', 'Two-color CP', 'folding', 'palette', 'TwoColoredTask'),
  notImplemented('Fold', 'Fold', 'folding', 'origami', 'FoldingServiceImpl.fold'),
  porting('FoldAnother', 'Another solution', 'folding', 'skip-forward', 'FoldingServiceImpl.foldAnother'),
  porting('DuplicateFoldedModel', 'Duplicate folded model', 'folding', 'copy', 'FoldingServiceImpl.duplicate'),
  notImplemented('Check1', 'Check 1', 'check-fix', 'badge-alert', 'Check1'),
  notImplemented('Check2', 'Check 2', 'check-fix', 'badge-alert', 'Check2'),
  notImplemented('Check3', 'Check 3', 'check-fix', 'badge-alert', 'Check3'),
  notImplemented('Check4', 'Check 4', 'check-fix', 'badge-alert', 'Check4'),
  notImplemented('Fix1', 'Fix 1', 'check-fix', 'wrench', 'Fix1'),
  notImplemented('Fix2', 'Fix 2', 'check-fix', 'wrench', 'Fix2'),
  notImplemented('OrganizeCircles', 'Organize circles', 'annotations', 'circle-ellipsis', 'OrganizeCircles'),
];

export const ORISTUDIO_CP_SOURCE_MAP_OPERATION_IDS = [
  'DrawCreaseFree',
  'MoveCreasePattern',
  'LineSegmentDelete',
  'ChangeCreaseType',
  'LengthenCrease',
  'SquareBisector',
  'Inward',
  'PerpendicularDraw',
  'SymmetricDraw',
  'DrawCreaseRestricted',
  'DrawCreaseSymmetric',
  'DrawCreaseAngleRestricted',
  'DrawPoint',
  'DeletePoint',
  'AngleSystem',
  'DrawCreaseAngleRestricted3',
  'CreaseSelect',
  'CreaseUnselect',
  'CreaseMove',
  'CreaseCopy',
  'CreaseMakeMountain',
  'CreaseMakeValley',
  'CreaseMakeEdge',
  'BackgroundChangePosition',
  'LineSegmentDivision',
  'LineSegmentRatioSet',
  'PolygonSetNoCorners',
  'CreaseAdvanceType',
  'CreaseMove4p',
  'CreaseCopy4p',
  'FishBoneDraw',
  'CreaseMakeMv',
  'DoubleSymmetricDraw',
  'CreasesAlternateMv',
  'DrawCreaseAngleRestricted5',
  'VertexMakeAngularlyFlatFoldable',
  'FoldableLineInput',
  'ParallelDraw',
  'VertexDeleteOnCrease',
  'CircleDraw',
  'CircleDrawThreePoint',
  'CircleDrawSeparate',
  'CircleDrawTangentLine',
  'CircleDrawInverted',
  'CircleDrawFree',
  'CircleDrawConcentric',
  'CircleDrawConcentricSelect',
  'CircleDrawConcentricTwoCircleSelect',
  'ParallelDrawWidth',
  'ContinuousSymmetricDraw',
  'DisplayLengthBetweenPoints1',
  'DisplayLengthBetweenPoints2',
  'DisplayAngleBetweenThreePoints1',
  'DisplayAngleBetweenThreePoints2',
  'DisplayAngleBetweenThreePoints3',
  'CreaseToggleMv',
  'CircleChangeColor',
  'CreaseMakeAux',
  'OperationFrameCreate',
  'VoronoiCreate',
  'FlatFoldableCheck',
  'CreaseDeleteOverlapping',
  'CreaseDeleteIntersecting',
  'SelectPolygon',
  'UnselectPolygon',
  'SelectLineIntersecting',
  'UnselectLineIntersecting',
  'LengthenCreaseSameColor',
  'FoldableLineDraw',
  'ReplaceLineTypeSelect',
  'DeleteLineTypeSelect',
  'SelectLasso',
  'UnselectLasso',
  'Text',
  'DrawBlintz',
  'DrawFishBase',
  'DrawDoveBase',
  'DrawBirdBase',
  'DrawFrogBase',
  'ModifyCalculatedShape',
  'MoveCalculatedShape',
  'ChangeStandardFace',
  'AddFoldingConstraint',
  'Axiom5',
  'Axiom7',
  'FixInaccurate',
  'ImportCp',
  'ExportCp',
  'ImportFold',
  'ExportFold',
  'ImportOri',
  'ExportOri',
  'ImportOrh',
  'ExportOrh',
  'ImportObj',
  'ExportDxf',
  'SaveConvert',
  'SaveVersionDetect',
  'CheckCamv',
  'FoldingEstimate',
  'FoldingEstimateSpecific',
  'FoldingEstimateSave100',
  'TwoColoredCp',
  'Fold',
  'FoldAnother',
  'DuplicateFoldedModel',
  'Check1',
  'Check2',
  'Check3',
  'Check4',
  'Fix1',
  'Fix2',
  'OrganizeCircles',
] as const;

export type OristudioCpOperationId = (typeof ORISTUDIO_CP_SOURCE_MAP_OPERATION_IDS)[number];

export function cpCommandsForGroup(
  group: OristudioCpCommandGroupId
): OristudioCpCommandDefinition[] {
  return ORISTUDIO_CP_COMMANDS.filter((command) => command.group === group);
}

export function cpCommandByOperation(
  operationId: OristudioCpOperationId
): OristudioCpCommandDefinition | undefined {
  return ORISTUDIO_CP_COMMANDS.find((command) => command.operationId === operationId);
}

export function cpRailCommands(): OristudioCpCommandDefinition[] {
  return ORISTUDIO_CP_COMMANDS.filter(
    (command) => command.placement === 'left-rail' || command.placement === 'left-rail-overflow'
  );
}
