import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type PointerEvent,
  type SetStateAction,
} from 'react';
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchRef } from 'react-zoom-pan-pinch';
import { ChevronDown, ChevronRight, GitBranch, Grid2X2, Magnet, ScanLine } from 'lucide-react';
import type {
  OristudioCpCommandPayload,
  OristudioCpCommandPreview,
  OristudioCpCircle,
  OristudioCpCustomLineType,
  OristudioCpDocumentSnapshot,
  OristudioCpLineColor,
  OristudioCpLineSegment,
  OristudioCpRgbColor,
} from '../../engine/oristudioCpTypes';
import { formatNumber, paperToSvg, type Point } from '../../lib/geometry';
import { getViewportFitScale, type ViewportSize } from '../../lib/designViewport';
import {
  cpActionById,
  type OristudioCpActionDefinition,
  type OristudioCpActionInputMode,
  type OristudioCpCommandActionDefinition,
} from '../../lib/oristudioCpActions';
import {
  cpCommandByOperation,
  type OristudioCpCommandDefinition,
} from '../../lib/oristudioCpCommands';
import {
  cancelOristudioCpToolState,
  IDLE_ORISTUDIO_CP_TOOL_STATE,
  transitionOristudioCpToolState,
} from '../../lib/oristudioCpToolState';
import {
  DEFAULT_ORISTUDIO_CP_TOOL_OPTIONS,
  ORISTUDIO_CP_CUSTOM_LINE_TYPE_OPTIONS,
  ORISTUDIO_CP_RATIO_PRESETS,
  ORISTUDIO_CP_REPLACE_TARGET_LINE_TYPE_OPTIONS,
  cpToolSettingGroupsForCommand,
  evaluateOrieditaRatioExpression,
  formatOrieditaRatioHalf,
  formatOrieditaRatioNumber,
  parseOrieditaRatioHalfInput,
  type OristudioCpRatioExpression,
  type OristudioCpToolOptions,
  type OristudioCpToolSettingGroup,
  ratioExpressionFromHalves,
  ratioHalvesFromExpression,
} from '../../lib/oristudioCpToolSettings';
import {
  CP_PAPER_RECT,
  CP_PAPER_SHADOW_RECT,
  CP_VIEWBOX_SIZE,
  CP_WORLD_RECT,
  cpLineAssignmentLabel,
  cpLineColorClass,
  cpSelectionSize,
  cpSvgPointToModel,
  getCpGridLines,
  getCpVertices,
  getEditableCpModelBounds,
  getOrieditaGridBasis,
  modelPointToCpSvg,
  nearestCpSnapTarget,
  nearestOrieditaDrawPointTarget,
  textCoordinate,
  visibleOrieditaGridMetadata,
  type CpSnapTarget,
  type CpVertex,
  type OristudioCpSelection,
} from '../../lib/creasePatternViewport';
import type { Selection, TreeProject } from '../../lib/sampleProject';
import {
  isCreaseSelected,
  isFacetSelected,
  selectionSize,
  toggleCreaseSelection,
  toggleFacetSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { IconButton } from '../ui/IconButton';
import { SegmentedControl } from '../ui/SegmentedControl';
import { CpToolRail } from './CpToolRail';
import { NextDocumentAction } from './NextDocumentAction';
import {
  isViewportInteractiveTarget,
  ViewportToolbar,
  ViewportToolbarSeparator,
} from './ViewportToolbar';

function creaseClass(fold: string, kind: string, mode: 'mvf' | 'agrh'): string {
  if (mode === 'agrh') return `crease crease--kind-${kind}`;
  return `crease crease--fold-${fold}`;
}

function formatZoom(scale: number): string {
  return `${Math.round(scale * 100)}%`;
}

function modelSelectionDistance(
  bounds: ReturnType<typeof getEditableCpModelBounds>,
  zoomScale = 1
): number {
  const baseDistance =
    (Math.max(bounds.spanX, bounds.spanY) / CP_PAPER_RECT.width) * 8;
  const zoomAdjustedDistance = zoomScale > 1 ? baseDistance / zoomScale : baseDistance;
  return Math.max(
    1e-6,
    zoomAdjustedDistance
  );
}

function cpCommandPayloadDefaults(
  command: OristudioCpCommandDefinition,
  bounds: ReturnType<typeof getEditableCpModelBounds>,
  gridWidth: number | undefined,
  lineColor: OristudioCpLineColor,
  zoomScale: number,
  toolOptions: OristudioCpToolOptions
): OristudioCpCommandPayload {
  const payload: OristudioCpCommandPayload = {};
  const operationId = command.operationId;

  if ((command.toolSteps?.length ?? 0) > 0 || command.inputMode === 'drag-path') {
    payload.selection_distance = modelSelectionDistance(bounds, zoomScale);
  }

  if (
    operationId === 'CreaseMakeMv' ||
    operationId === 'CreasesAlternateMv' ||
    operationId === 'LengthenCrease' ||
    operationId === 'DrawCreaseFree' ||
    operationId === 'DrawCreaseRestricted' ||
    operationId === 'DrawCreaseSymmetric' ||
    operationId === 'DrawCreaseAngleRestricted' ||
    operationId === 'DrawCreaseAngleRestricted3' ||
    operationId === 'DrawCreaseAngleRestricted5' ||
    operationId === 'SquareBisector' ||
    operationId === 'Inward' ||
    operationId === 'PerpendicularDraw' ||
    operationId === 'SymmetricDraw' ||
    operationId === 'FishBoneDraw' ||
    operationId === 'DoubleSymmetricDraw' ||
    operationId === 'VertexMakeAngularlyFlatFoldable' ||
    operationId === 'FoldableLineInput' ||
    operationId === 'ParallelDraw' ||
    operationId === 'ParallelDrawWidth' ||
    operationId === 'ContinuousSymmetricDraw' ||
    operationId === 'FoldableLineDraw' ||
    operationId === 'Axiom5' ||
    operationId === 'Axiom7' ||
    operationId === 'PolygonSetNoCorners' ||
    operationId === 'DrawBlintz' ||
    operationId === 'DrawFishBase' ||
    operationId === 'DrawDoveBase' ||
    operationId === 'DrawBirdBase' ||
    operationId === 'DrawFrogBase'
  ) {
    payload.line_color = lineColor;
  }

  if (
    operationId === 'FishBoneDraw' ||
    operationId === 'VertexMakeAngularlyFlatFoldable' ||
    operationId === 'FoldableLineInput' ||
    operationId === 'FoldableLineDraw'
  ) {
    payload.grid_width = gridWidth;
  }

  if (
    operationId === 'AngleSystem' ||
    operationId === 'DrawCreaseAngleRestricted' ||
    operationId === 'DrawCreaseAngleRestricted3' ||
    operationId === 'DrawCreaseAngleRestricted5'
  ) {
    payload.angle_system_divider = toolOptions.angleSystemDivider;
    payload.angles = toolOptions.angleSystemAngles;
  }

  if (operationId === 'LineSegmentDivision') {
    payload.division_count = toolOptions.divisionCount;
  }

  if (operationId === 'LineSegmentRatioSet') {
    const ratio = evaluateOrieditaRatioExpression(toolOptions.divisionRatio);
    payload.ratio_s = ratio.ratioS;
    payload.ratio_t = ratio.ratioT;
  }

  if (operationId === 'PolygonSetNoCorners') {
    payload.polygon_corners = toolOptions.polygonCorners;
  }

  if (operationId === 'CircleChangeColor') {
    payload.custom_circle_color = toolOptions.customCircleColor;
  }

  if (operationId === 'ParallelDrawWidth') {
    payload.width = toolOptions.parallelWidth;
  }

  if (
    toolOptions.candidateIndex !== null &&
    cpToolSettingGroupsForCommand(command).includes('candidate-choice')
  ) {
    payload.candidate_index = toolOptions.candidateIndex;
  }

  if (operationId === 'ReplaceLineTypeSelect') {
    payload.custom_from_line_type = toolOptions.customFromLineType;
    payload.custom_to_line_type = toolOptions.customToLineType;
  }

  if (operationId === 'DeleteLineTypeSelect') {
    payload.custom_line_type = toolOptions.customLineType;
  }

  if (operationId === 'FixInaccurate') {
    payload.fix_precision = toolOptions.fixPrecision;
    payload.fix_precision_use_bp = toolOptions.fixPrecisionUseBp;
    payload.fix_precision_use_22_5 = toolOptions.fixPrecisionUse22_5;
  }

  return payload;
}

function pointDistanceSquared(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return dx * dx + dy * dy;
}

function isLineClickSelectionOperation(operationId: string | null | undefined): boolean {
  return operationId === 'CreaseSelect' || operationId === 'CreaseUnselect';
}

function isRestrictedDrawOperation(operationId: string | null | undefined): boolean {
  return operationId === 'DrawCreaseRestricted';
}

function cpLineTypeStatusLabel(lineColor: OristudioCpLineColor): string {
  switch (lineColor) {
    case 'Red1':
      return 'Line M';
    case 'Blue2':
      return 'Line V';
    case 'Black0':
      return 'Line E';
    case 'Cyan3':
      return 'Line A';
    default:
      return `Line ${cpLineAssignmentLabel(lineColor)}`;
  }
}

function activeActionInputMode(
  action: OristudioCpActionDefinition | undefined,
  command: OristudioCpCommandDefinition | undefined
): OristudioCpActionInputMode | undefined {
  if (action?.kind === 'command') return action.inputMode ?? action.command.inputMode;
  return command?.inputMode;
}

function cpCommandRequiresContextApply(command: OristudioCpCommandDefinition): boolean {
  if ((command.toolSteps?.length ?? 0) > 0) return false;
  return cpToolSettingGroupsForCommand(command).some(
    (group) => group !== 'line-color' && group !== 'line-select-help'
  );
}

function isCpLineEventTarget(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest('[data-cp-line-id], [data-cp-line-hit-id]') !== null
  );
}

type CpMeasurementSlotId = 'length1' | 'length2' | 'angle1' | 'angle2' | 'angle3';
type CpMeasurementSlots = Record<CpMeasurementSlotId, number | null>;

const CP_MEASUREMENT_SLOT_LABELS: Record<CpMeasurementSlotId, string> = {
  length1: 'L1',
  length2: 'L2',
  angle1: 'A1',
  angle2: 'A2',
  angle3: 'A3',
};

const CP_MEASUREMENT_SLOT_ORDER: readonly CpMeasurementSlotId[] = [
  'length1',
  'length2',
  'angle1',
  'angle2',
  'angle3',
];

function createEmptyCpMeasurementSlots(): CpMeasurementSlots {
  return {
    length1: null,
    length2: null,
    angle1: null,
    angle2: null,
    angle3: null,
  };
}

function cpMeasurementSlotForOperation(
  operationId: OristudioCpCommandDefinition['operationId'] | null | undefined
): CpMeasurementSlotId | null {
  switch (operationId) {
    case 'DisplayLengthBetweenPoints1':
      return 'length1';
    case 'DisplayLengthBetweenPoints2':
      return 'length2';
    case 'DisplayAngleBetweenThreePoints1':
      return 'angle1';
    case 'DisplayAngleBetweenThreePoints2':
      return 'angle2';
    case 'DisplayAngleBetweenThreePoints3':
      return 'angle3';
    default:
      return null;
  }
}

function isCpMeasurementOperation(
  operationId: OristudioCpCommandDefinition['operationId'] | null | undefined
): boolean {
  return cpMeasurementSlotForOperation(operationId) !== null;
}

function computeCpMeasurementValue(
  operationId: OristudioCpCommandDefinition['operationId'],
  points: readonly Point[]
): number | null {
  const slot = cpMeasurementSlotForOperation(operationId);
  if (!slot) return null;

  if (slot === 'length1' || slot === 'length2') {
    const [a, b] = points;
    if (!a || !b) return null;
    return Math.hypot(b.x - a.x, b.y - a.y);
  }

  const [a, center, b] = points;
  if (!a || !center || !b) return null;
  const start = Math.atan2(a.y - center.y, a.x - center.x);
  const end = Math.atan2(b.y - center.y, b.x - center.x);
  const degrees = ((end - start) * 180) / Math.PI;
  return ((degrees % 360) + 360) % 360;
}

function formatCpMeasurementValue(slot: CpMeasurementSlotId, value: number | null): string {
  if (value === null) return '-';
  const precision = slot.startsWith('angle') ? 2 : 3;
  const unit = slot.startsWith('angle') ? ' deg' : '';
  return `${formatNumber(value, precision)}${unit}`;
}

export function CreasePatternPanel() {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const transformRef = useRef<ReactZoomPanPinchRef | null>(null);
  const [zoomPercent, setZoomPercent] = useState(100);
  const [spacePressed, setSpacePressed] = useState(false);
  const [cursorModelPoint, setCursorModelPoint] = useState<Point | null>(null);
  const [snapTarget, setSnapTarget] = useState<CpSnapTarget | null>(null);
  const [cpToolState, setCpToolState] = useState(IDLE_ORISTUDIO_CP_TOOL_STATE);
  const [activeCpLineColor, setActiveCpLineColor] = useState<OristudioCpLineColor>('Red1');
  const [cpToolOptions, setCpToolOptions] = useState<OristudioCpToolOptions>(
    DEFAULT_ORISTUDIO_CP_TOOL_OPTIONS
  );
  const [cpToolPoints, setCpToolPoints] = useState<Point[]>([]);
  const [cpToolPath, setCpToolPath] = useState<Point[]>([]);
  const [cpMeasurementSlots, setCpMeasurementSlots] = useState<CpMeasurementSlots>(
    createEmptyCpMeasurementSlots
  );
  const [cpCommandPreview, setCpCommandPreview] = useState<OristudioCpCommandPreview | null>(null);
  const cpPreviewRequestRef = useRef(0);
  const cpToolDragRef = useRef<{
    operationId: OristudioCpCommandDefinition['operationId'];
    actionId: OristudioCpCommandActionDefinition['id'] | null;
    mode: 'drag-line' | 'drag-path';
    pointerId: number;
    points: Point[];
  } | null>(null);

  const project = useWorkspaceStore((state) => state.project);
  const status = useWorkspaceStore((state) => state.status);
  const error = useWorkspaceStore((state) => state.error);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const importedCreasePattern = useWorkspaceStore((state) => state.importedCreasePattern);
  const oristudioCpDocument = useWorkspaceStore((state) => state.oristudioCpDocument);
  const oristudioCpError = useWorkspaceStore((state) => state.oristudioCpError);
  const oristudioCpSelection = useWorkspaceStore((state) => state.oristudioCpSelection);
  const oristudioCpViewport = useWorkspaceStore((state) => state.oristudioCpViewport);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);
  const mode = useWorkspaceStore((state) => state.creaseColorMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const setMode = useWorkspaceStore((state) => state.setCreaseColorMode);
  const select = useWorkspaceStore((state) => state.select);
  const setOristudioCpViewportOption = useWorkspaceStore(
    (state) => state.setOristudioCpViewportOption
  );
  const toggleOristudioCpLineSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpLineSelection
  );
  const toggleOristudioCpVertexSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpVertexSelection
  );
  const toggleOristudioCpPointSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpPointSelection
  );
  const toggleOristudioCpCircleSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpCircleSelection
  );
  const toggleOristudioCpTextSelection = useWorkspaceStore(
    (state) => state.toggleOristudioCpTextSelection
  );
  const clearOristudioCpSelection = useWorkspaceStore((state) => state.clearOristudioCpSelection);
  const executeOristudioCpCommand = useWorkspaceStore(
    (state) => state.executeOristudioCpCommand
  );
  const previewOristudioCpCommand = useWorkspaceStore(
    (state) => state.previewOristudioCpCommand
  );

  const editableCp = documentMode === 'crease-pattern' ? oristudioCpDocument?.document : null;
  const editableCpHandle =
    documentMode === 'crease-pattern' ? (oristudioCpDocument?.handle ?? null) : null;
  const editableCpSummary = oristudioCpDocument?.summary ?? null;
  const editableCpBounds = useMemo(() => getEditableCpModelBounds(editableCp), [editableCp]);
  const editableCpVisibleGrid = useMemo(
    () =>
      editableCp && oristudioCpViewport.gridVisible
        ? visibleOrieditaGridMetadata(editableCp.crease_pattern.grid)
        : null,
    [editableCp, oristudioCpViewport.gridVisible]
  );
  const editableCpGridLines = useMemo(
    () => (editableCpVisibleGrid ? getCpGridLines(editableCpBounds, editableCpVisibleGrid) : []),
    [editableCpBounds, editableCpVisibleGrid]
  );
  const editableCpGridWidth = useMemo(
    () =>
      editableCp
        ? getOrieditaGridBasis(visibleOrieditaGridMetadata(editableCp.crease_pattern.grid)).gridWidth
        : undefined,
    [editableCp]
  );
  const editableCpVertices = useMemo(() => getCpVertices(editableCp), [editableCp]);
  const hasEditableCreasePattern =
    !!editableCp &&
    (editableCp.crease_pattern.line_segments.length > 0 ||
      editableCp.crease_pattern.circles.length > 0 ||
      editableCp.crease_pattern.points.length > 0 ||
      editableCp.crease_pattern.texts.length > 0);
  const hasCreasePattern =
    hasEditableCreasePattern || project.creases.length > 0 || project.facets.length > 0;
  const editableSelectionSize = cpSelectionSize(oristudioCpSelection);
  const activeCpAction = useMemo(
    () => (cpToolState.activeActionId ? cpActionById(cpToolState.activeActionId) : undefined),
    [cpToolState.activeActionId]
  );
  const activeCpCommand = useMemo(
    () => {
      if (activeCpAction?.kind === 'command') return activeCpAction.command;
      return cpToolState.activeOperationId
        ? cpCommandByOperation(cpToolState.activeOperationId)
        : undefined;
    },
    [activeCpAction, cpToolState.activeOperationId]
  );
  const activeCpInputMode = useMemo(
    () => activeActionInputMode(activeCpAction, activeCpCommand),
    [activeCpAction, activeCpCommand]
  );
  const liveCommandPreviewPoints = useMemo(() => {
    if (cpToolPath.length > 0) return cpToolPath;
    if (!activeCpCommand || cpToolState.phase !== 'active') return cpToolPoints;
    const stepCount = activeCpCommand.toolSteps?.length ?? 0;
    const livePoint = snapTarget?.point ?? cursorModelPoint;
    if (stepCount === 0 || !livePoint || cpToolPoints.length === 0) return cpToolPoints;
    return [...cpToolPoints, livePoint].slice(0, stepCount);
  }, [activeCpCommand, cpToolPath, cpToolPoints, cpToolState.phase, cursorModelPoint, snapTarget]);
  const localDragLinePreviewSegments = useMemo<OristudioCpLineSegment[]>(() => {
    if (activeCpInputMode !== 'drag-line' || liveCommandPreviewPoints.length < 2) return [];
    const a = liveCommandPreviewPoints[0];
    const b = liveCommandPreviewPoints[1];
    if (!a || !b) return [];
    return [
      {
        a,
        b,
        color: activeCpLineColor,
        active: 'Inactive0',
        selected: 0,
        customized: 0,
        customized_color: { red: 0, green: 0, blue: 0 },
      },
    ];
  }, [activeCpInputMode, activeCpLineColor, liveCommandPreviewPoints]);
  const renderedCommandPreviewPoints =
    activeCpInputMode === 'drag-line' ? [] : liveCommandPreviewPoints;
  const renderedCommandPreviewSegments =
    localDragLinePreviewSegments.length > 0
      ? localDragLinePreviewSegments
      : (cpCommandPreview?.segments ?? []);
  const renderedCommandPreviewCircles = cpCommandPreview?.circles ?? [];
  const buildCpCommandPayload = useCallback(
    (
      command: OristudioCpCommandDefinition,
      payload: OristudioCpCommandPayload = {}
    ): OristudioCpCommandPayload => ({
      ...cpCommandPayloadDefaults(
        command,
        editableCpBounds,
        editableCpGridWidth,
        activeCpLineColor,
        zoomPercent / 100,
        cpToolOptions
      ),
      ...payload,
    }),
    [activeCpLineColor, cpToolOptions, editableCpBounds, editableCpGridWidth, zoomPercent]
  );

  useEffect(() => {
    if (
      !editableCp ||
      !activeCpCommand ||
      activeCpCommand.uiStatus !== 'ready' ||
      cpToolState.phase !== 'active' ||
      isCpMeasurementOperation(activeCpCommand.operationId) ||
      activeCpInputMode === 'drag-path' ||
      activeCpInputMode === 'drag-line' ||
      liveCommandPreviewPoints.length === 0
    ) {
      cpPreviewRequestRef.current += 1;
      setCpCommandPreview(null);
      return;
    }

    const requestId = cpPreviewRequestRef.current + 1;
    cpPreviewRequestRef.current = requestId;
    void previewOristudioCpCommand(
      activeCpCommand.operationId,
      buildCpCommandPayload(activeCpCommand, {
        line_ids: oristudioCpSelection.lines,
        points: liveCommandPreviewPoints,
      })
    ).then((preview) => {
      if (cpPreviewRequestRef.current === requestId) {
        setCpCommandPreview(preview);
      }
    });
  }, [
    activeCpCommand,
    activeCpInputMode,
    buildCpCommandPayload,
    cpToolState.phase,
    editableCp,
    liveCommandPreviewPoints,
    oristudioCpSelection.lines,
    previewOristudioCpCommand,
  ]);

  const handleCpToolAction = useCallback(
    (action: OristudioCpActionDefinition) => {
      if (action.kind === 'line-type') {
        setActiveCpLineColor(action.lineColor);
        return;
      }

      const command = action.command;
      setCpToolPoints([]);
      setCpToolPath([]);
      setCpCommandPreview(null);
      cpToolDragRef.current = null;
      setCpToolState((state) =>
        transitionOristudioCpToolState(state, {
          type: 'selectAction',
          action,
          editable: !!editableCp,
        })
      );

      if (!editableCp || command.uiStatus !== 'ready' || (command.toolSteps?.length ?? 0) > 0) {
        return;
      }

      if (cpCommandRequiresContextApply(command)) {
        return;
      }

      void (async () => {
        const succeeded = await executeOristudioCpCommand(
          command.operationId,
          buildCpCommandPayload(command, {
            line_ids: oristudioCpSelection.lines,
          })
        );
        setCpToolPoints([]);
        setCpToolState((state) =>
          state.activeActionId === action.id
            ? transitionOristudioCpToolState(
                state,
                succeeded
                  ? { type: 'commit' }
                  : {
                      type: 'commandError',
                      message: useWorkspaceStore.getState().oristudioCpError ?? 'Command failed',
                    }
              )
            : state
        );
      })();
    },
    [buildCpCommandPayload, editableCp, executeOristudioCpCommand, oristudioCpSelection.lines]
  );

  const handleApplyActiveContextCommand = useCallback(() => {
    if (
      !editableCp ||
      !activeCpCommand ||
      activeCpCommand.uiStatus !== 'ready' ||
      (activeCpCommand.toolSteps?.length ?? 0) > 0
    ) {
      return;
    }

    void (async () => {
      const selectionPayload: OristudioCpCommandPayload = {
        line_ids: oristudioCpSelection.lines,
      };
      if (activeCpCommand.operationId === 'CircleChangeColor') {
        selectionPayload.circle_ids = oristudioCpSelection.circles;
      }
      const succeeded = await executeOristudioCpCommand(
        activeCpCommand.operationId,
        buildCpCommandPayload(activeCpCommand, selectionPayload)
      );
      setCpToolState((state) =>
        state.activeOperationId === activeCpCommand.operationId
          ? transitionOristudioCpToolState(
              state,
              succeeded
                ? { type: 'commit' }
                : {
                    type: 'commandError',
                    message: useWorkspaceStore.getState().oristudioCpError ?? 'Command failed',
                  }
            )
          : state
      );
    })();
  }, [
    activeCpCommand,
    buildCpCommandPayload,
    editableCp,
    executeOristudioCpCommand,
    oristudioCpSelection.circles,
    oristudioCpSelection.lines,
  ]);

  const eventToEditableModelPoint = useCallback(
    (event: PointerEvent<SVGElement>): Point | null => {
      const svg = svgRef.current;
      if (!svg || !editableCp) return null;
      const bounds = svg.getBoundingClientRect();
      if (bounds.width <= 0 || bounds.height <= 0) return null;
      const svgPoint = {
        x: ((event.clientX - bounds.left) / bounds.width) * CP_VIEWBOX_SIZE,
        y: ((event.clientY - bounds.top) / bounds.height) * CP_VIEWBOX_SIZE,
      };
      return cpSvgPointToModel(svgPoint, editableCpBounds);
    },
    [editableCp, editableCpBounds]
  );

  const resolveEditableToolPoint = useCallback(
    (event: PointerEvent<SVGElement>): Point | null => {
      if (!editableCp) return null;
      const modelPoint = eventToEditableModelPoint(event);
      if (!modelPoint) return null;
      const selectionDistance = modelSelectionDistance(editableCpBounds, zoomPercent / 100);
      const target = nearestCpSnapTarget(
        editableCp,
        modelPoint,
        editableCpBounds,
        oristudioCpViewport,
        selectionDistance
      );
      return target?.point ?? modelPoint;
    },
    [editableCp, editableCpBounds, eventToEditableModelPoint, oristudioCpViewport, zoomPercent]
  );

  const resolveEditableDrawPoint = useCallback(
    (
      event: PointerEvent<SVGElement>,
      requireSnap: boolean
    ): { point: Point; target: CpSnapTarget | null } | null => {
      if (!editableCp) return null;
      const modelPoint = eventToEditableModelPoint(event);
      if (!modelPoint) return null;
      const target = nearestOrieditaDrawPointTarget(
        editableCp,
        modelPoint,
        editableCpBounds,
        oristudioCpViewport,
        modelSelectionDistance(editableCpBounds, zoomPercent / 100)
      );
      if (!target && requireSnap) return null;
      return { point: target?.point ?? modelPoint, target };
    },
    [editableCp, editableCpBounds, eventToEditableModelPoint, oristudioCpViewport, zoomPercent]
  );

  const updateEditablePointerStatus = useCallback(
    (event: PointerEvent<SVGElement>) => {
      if (!editableCp) return;
      const modelPoint = eventToEditableModelPoint(event);
      setCursorModelPoint(modelPoint);
      if (modelPoint && activeCpInputMode === 'drag-line') {
        setSnapTarget(
          nearestOrieditaDrawPointTarget(
            editableCp,
            modelPoint,
            editableCpBounds,
            oristudioCpViewport,
            modelSelectionDistance(editableCpBounds, zoomPercent / 100)
          )
        );
        return;
      }
      setSnapTarget(
        modelPoint
          ? nearestCpSnapTarget(
              editableCp,
              modelPoint,
              editableCpBounds,
              oristudioCpViewport,
              modelSelectionDistance(editableCpBounds, zoomPercent / 100)
            )
          : null
      );
    },
    [
      activeCpInputMode,
      editableCp,
      editableCpBounds,
      eventToEditableModelPoint,
      oristudioCpViewport,
      zoomPercent,
    ]
  );

  const handleEditableToolPointerDown = useCallback(
    (event: PointerEvent<SVGElement>) => {
      if (
        event.button !== 0 ||
        spacePressed ||
        !editableCp ||
        !activeCpCommand ||
        activeCpCommand.uiStatus !== 'ready' ||
        cpToolState.phase !== 'active'
      ) {
        return;
      }
      const stepCount = activeCpCommand.toolSteps?.length ?? 0;
      if (stepCount === 0) return;
      if (
        isLineClickSelectionOperation(activeCpCommand.operationId) &&
        isCpLineEventTarget(event.target)
      ) {
        return;
      }

      if (activeCpInputMode === 'drag-line') {
        const resolved = resolveEditableDrawPoint(
          event,
          isRestrictedDrawOperation(activeCpCommand.operationId)
        );
        if (!resolved) return;
        event.preventDefault();
        event.stopPropagation();
        cpToolDragRef.current = {
          operationId: activeCpCommand.operationId,
          actionId: activeCpAction?.kind === 'command' ? activeCpAction.id : null,
          mode: 'drag-line',
          pointerId: event.pointerId,
          points: [resolved.point],
        };
        if (typeof event.pointerId === 'number') {
          event.currentTarget.setPointerCapture?.(event.pointerId);
        }
        setSnapTarget(resolved.target);
        setCpToolPoints([resolved.point]);
        setCpToolPath([resolved.point]);
        return;
      }

      if (activeCpInputMode === 'drag-path') {
        const point = eventToEditableModelPoint(event);
        if (!point) return;
        event.preventDefault();
        event.stopPropagation();
        cpToolDragRef.current = {
          operationId: activeCpCommand.operationId,
          actionId: activeCpAction?.kind === 'command' ? activeCpAction.id : null,
          mode: 'drag-path',
          pointerId: event.pointerId,
          points: [point],
        };
        if (typeof event.pointerId === 'number') {
          event.currentTarget.setPointerCapture?.(event.pointerId);
        }
        setCpToolPath([point]);
        return;
      }

      const point = resolveEditableToolPoint(event);
      if (!point) return;

      event.preventDefault();
      event.stopPropagation();
      const nextPoints = [...cpToolPoints, point];

      if (nextPoints.length < stepCount) {
        setCpToolPoints(nextPoints);
        setCpToolState((state) =>
          state.activeOperationId === activeCpCommand.operationId
            ? transitionOristudioCpToolState(state, { type: 'advanceStep' })
            : state
        );
        return;
      }

      const measurementSlot = cpMeasurementSlotForOperation(activeCpCommand.operationId);
      if (measurementSlot) {
        const value = computeCpMeasurementValue(activeCpCommand.operationId, nextPoints);
        if (value === null) return;
        setCpMeasurementSlots((current) => ({
          ...current,
          [measurementSlot]: value,
        }));
        setCpToolPoints([]);
        setCpToolState((state) =>
          state.activeOperationId === activeCpCommand.operationId
            ? transitionOristudioCpToolState(state, { type: 'commit', keepActive: true })
            : state
        );
        return;
      }

      setCpToolPoints([]);
      void (async () => {
        const succeeded = await executeOristudioCpCommand(
          activeCpCommand.operationId,
          buildCpCommandPayload(activeCpCommand, {
            line_ids: oristudioCpSelection.lines,
            points: nextPoints,
          })
        );
        setCpToolState((state) =>
          state.activeOperationId === activeCpCommand.operationId
            ? transitionOristudioCpToolState(
                state,
                succeeded
                  ? { type: 'commit' }
                  : {
                      type: 'commandError',
                      message: useWorkspaceStore.getState().oristudioCpError ?? 'Command failed',
                    }
              )
            : state
        );
      })();
    },
    [
      activeCpAction,
      activeCpCommand,
      activeCpInputMode,
      buildCpCommandPayload,
      cpToolPoints,
      cpToolState.phase,
      editableCp,
      eventToEditableModelPoint,
      executeOristudioCpCommand,
      oristudioCpSelection.lines,
      resolveEditableDrawPoint,
      resolveEditableToolPoint,
      spacePressed,
    ]
  );

  const handleEditablePointerMove = useCallback(
    (event: PointerEvent<SVGElement>) => {
      updateEditablePointerStatus(event);
      const drag = cpToolDragRef.current;
      if (!drag || drag.pointerId !== event.pointerId) return;
      if (drag.mode === 'drag-line') {
        const resolved = resolveEditableDrawPoint(event, false);
        const startPoint = drag.points[0];
        if (!resolved || !startPoint) return;
        drag.points = [startPoint, resolved.point];
        setSnapTarget(resolved.target);
        setCpToolPath(drag.points);
        return;
      }
      const point = eventToEditableModelPoint(event);
      if (!point) return;
      const last = drag.points.at(-1);
      if (
        last &&
        pointDistanceSquared(last, point) <
          modelSelectionDistance(editableCpBounds, zoomPercent / 100) ** 2 / 16
      ) {
        return;
      }
      drag.points = [...drag.points, point];
      setCpToolPath(drag.points);
    },
    [
      editableCpBounds,
      eventToEditableModelPoint,
      resolveEditableDrawPoint,
      updateEditablePointerStatus,
      zoomPercent,
    ]
  );

  const finishEditableDragPath = useCallback(
    (event: PointerEvent<SVGElement>) => {
      const drag = cpToolDragRef.current;
      if (!drag || drag.pointerId !== event.pointerId) return;
      event.preventDefault();
      event.stopPropagation();
      const command = cpCommandByOperation(drag.operationId);
      if (!command) {
        cpToolDragRef.current = null;
        setCpToolPath([]);
        setCpToolPoints([]);
        return;
      }
      const finalResolution =
        drag.mode === 'drag-line'
          ? resolveEditableDrawPoint(event, isRestrictedDrawOperation(drag.operationId))
          : null;
      const finalPoint =
        drag.mode === 'drag-line' ? finalResolution?.point : eventToEditableModelPoint(event);
      const points =
        drag.mode === 'drag-line'
          ? drag.points[0] && finalPoint
            ? [drag.points[0], finalPoint]
            : drag.points.slice(0, 1)
          : finalPoint &&
              !drag.points.some((point) => pointDistanceSquared(point, finalPoint) < 1e-12)
            ? [...drag.points, finalPoint]
            : drag.points;
      cpToolDragRef.current = null;
      if (typeof event.pointerId === 'number') {
        event.currentTarget.releasePointerCapture?.(event.pointerId);
      }
      setCpToolPath([]);
      setCpToolPoints([]);
      if (points.length < 2) {
        setCpToolState((state) =>
          state.activeOperationId === command.operationId
            ? transitionOristudioCpToolState(state, {
                type: 'cancel',
                keepActive: drag.mode === 'drag-line',
              })
            : state
        );
        return;
      }

      void (async () => {
        const action = drag.actionId ? cpActionById(drag.actionId) : undefined;
        const succeeded = await executeOristudioCpCommand(
          command.operationId,
          buildCpCommandPayload(command, {
            line_ids: oristudioCpSelection.lines,
            points,
          })
        );
        setCpToolState((state) =>
          state.activeOperationId === command.operationId
            ? transitionOristudioCpToolState(
                state,
                succeeded
                  ? {
                      type: 'commit',
                      keepActive: action?.kind === 'command' ? action.repeatable : false,
                    }
                  : {
                      type: 'commandError',
                      message: useWorkspaceStore.getState().oristudioCpError ?? 'Command failed',
                    }
              )
            : state
        );
      })();
    },
    [
      buildCpCommandPayload,
      eventToEditableModelPoint,
      executeOristudioCpCommand,
      oristudioCpSelection.lines,
      resolveEditableDrawPoint,
    ]
  );

  const cancelEditableDragPath = useCallback((event: PointerEvent<SVGElement>) => {
    const drag = cpToolDragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    cpToolDragRef.current = null;
    setCpToolPoints([]);
    setCpToolPath([]);
  }, []);

  const handleEditableLineClick = useCallback(
    (id: number, additive = false) => {
      if (
        activeCpCommand?.uiStatus === 'ready' &&
        cpToolState.phase === 'active' &&
        isLineClickSelectionOperation(activeCpCommand.operationId)
      ) {
        setCpToolPoints([]);
        setCpToolPath([]);
        void (async () => {
          const succeeded = await executeOristudioCpCommand(
            activeCpCommand.operationId,
            buildCpCommandPayload(activeCpCommand, {
              line_ids: [id],
            })
          );
          setCpToolState((state) =>
            state.activeOperationId === activeCpCommand.operationId
              ? transitionOristudioCpToolState(
                  state,
                  succeeded
                    ? { type: 'commit' }
                    : {
                        type: 'commandError',
                        message: useWorkspaceStore.getState().oristudioCpError ?? 'Command failed',
                      }
                )
              : state
          );
        })();
        return;
      }

      if (cpToolState.phase === 'active') return;
      toggleOristudioCpLineSelection(id, additive);
    },
    [
      activeCpCommand,
      buildCpCommandPayload,
      cpToolState.phase,
      executeOristudioCpCommand,
      toggleOristudioCpLineSelection,
    ]
  );

  const handleEditableVertexClick = useCallback(
    (id: string, additive = false) => {
      if (cpToolState.phase === 'active') return;
      toggleOristudioCpVertexSelection(id, additive);
    },
    [cpToolState.phase, toggleOristudioCpVertexSelection]
  );

  const handleEditablePointClick = useCallback(
    (id: number, additive = false) => {
      if (cpToolState.phase === 'active') return;
      toggleOristudioCpPointSelection(id, additive);
    },
    [cpToolState.phase, toggleOristudioCpPointSelection]
  );

  const handleEditableCircleClick = useCallback(
    (id: number, additive = false) => {
      if (cpToolState.phase === 'active') return;
      toggleOristudioCpCircleSelection(id, additive);
    },
    [cpToolState.phase, toggleOristudioCpCircleSelection]
  );

  const handleEditableTextClick = useCallback(
    (id: number, additive = false) => {
      if (cpToolState.phase === 'active') return;
      toggleOristudioCpTextSelection(id, additive);
    },
    [cpToolState.phase, toggleOristudioCpTextSelection]
  );

  const clearSelectionOnBackgroundPointerDown = (event: PointerEvent<SVGElement>) => {
    if (event.button !== 0 || spacePressed) return;
    if (editableCp && editableSelectionSize > 0) {
      clearOristudioCpSelection();
      return;
    }
    if (selectionSize(selection) === 0) return;
    select({ kind: 'tree' });
  };
  const emptyStatusLabel =
    status === 'building_crease_pattern'
      ? 'Building crease pattern'
      : status === 'optimizing'
        ? 'Optimizing scale'
        : status === 'error' && error
          ? shortStatus(error.message)
          : documentMode === 'crease-pattern'
            ? 'No imported crease pattern'
            : 'No crease pattern';
  const sourceLabel =
    documentMode === 'crease-pattern' && importedCreasePattern
      ? [
          importedCreasePattern.source.filename,
          importedCreasePattern.lineOnly ? 'View only' : 'Simulatable',
          oristudioCpDocument
            ? `Editable kernel: ${oristudioCpDocument.summary.line_segments} lines`
            : oristudioCpError
              ? `Kernel unavailable: ${shortStatus(oristudioCpError)}`
              : 'Editable kernel pending',
        ].join(' | ')
      : null;

  const getViewportSize = useCallback((): ViewportSize | null => {
    const viewport = containerRef.current;
    if (!viewport) return null;
    return {
      width: viewport.clientWidth || viewport.offsetWidth,
      height: viewport.clientHeight || viewport.offsetHeight,
    };
  }, []);

  const computeFitScale = useCallback(() => {
    const viewport = getViewportSize();
    if (!viewport) return 1;
    return getViewportFitScale(viewport, CP_WORLD_RECT);
  }, [getViewportSize]);

  const fitToView = useCallback(
    (animationTime = 180) => {
      transformRef.current?.centerView(computeFitScale(), animationTime);
    },
    [computeFitScale]
  );

  const setActualSize = useCallback(() => {
    transformRef.current?.centerView(1, 160);
  }, []);

  const setZoomLevel = useCallback((scale: number) => {
    transformRef.current?.centerView(scale, 160);
  }, []);

  const clearEditablePointerStatus = useCallback(() => {
    setCursorModelPoint(null);
    setSnapTarget(null);
  }, []);

  const creasePatternFitKey = useMemo(
    () =>
      `${projectLoadId}:${project.creases.length}:${project.facets.length}:${editableCpSummary?.line_segments ?? 0}`,
    [editableCpSummary?.line_segments, project.creases.length, project.facets.length, projectLoadId]
  );
  const lastFittedCreasePatternRef = useRef<string | null>(null);

  const fitLoadedCreasePattern = useCallback(
    (animationTime = 0) => {
      if (!hasCreasePattern) {
        lastFittedCreasePatternRef.current = null;
        return true;
      }
      if (lastFittedCreasePatternRef.current === creasePatternFitKey) return true;
      const container = containerRef.current;
      if (!container || !transformRef.current || container.clientWidth <= 0 || container.clientHeight <= 0) {
        return false;
      }
      transformRef.current.centerView(computeFitScale(), animationTime);
      lastFittedCreasePatternRef.current = creasePatternFitKey;
      return true;
    },
    [computeFitScale, creasePatternFitKey, hasCreasePattern]
  );

  const fitLoadedCreasePatternRef = useRef(fitLoadedCreasePattern);
  useEffect(() => {
    fitLoadedCreasePatternRef.current = fitLoadedCreasePattern;
  }, [fitLoadedCreasePattern]);

  useEffect(() => {
    const container = containerRef.current;
    if (!hasCreasePattern) {
      lastFittedCreasePatternRef.current = null;
      return undefined;
    }

    let frame = requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
    const observer =
      typeof ResizeObserver === 'undefined' || !container
        ? null
        : new ResizeObserver(() => {
            if (lastFittedCreasePatternRef.current !== creasePatternFitKey) {
              cancelAnimationFrame(frame);
              frame = requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
            }
          });

    if (observer && container) {
      observer.observe(container);
    }
    return () => {
      cancelAnimationFrame(frame);
      observer?.disconnect();
    };
  }, [creasePatternFitKey, hasCreasePattern]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !hasCreasePattern) return undefined;

    const onKeyDown = (event: KeyboardEvent) => {
      const interactive = isViewportInteractiveTarget(event.target);
      if (event.key === 'Escape' && editableCp) {
        const cancellation = cancelOristudioCpToolState(cpToolState);
        if (cancellation.handled) {
          event.preventDefault();
          setCpToolPoints([]);
          setCpToolPath([]);
          cpToolDragRef.current = null;
          setCpToolState(cancellation.state);
          return;
        }
        if (editableSelectionSize > 0) {
          event.preventDefault();
          clearOristudioCpSelection();
          return;
        }
      }

      if (event.key === ' ' && !interactive) {
        event.preventDefault();
        setSpacePressed(true);
        return;
      }

      if (interactive || (!event.metaKey && !event.ctrlKey)) return;

      switch (event.key) {
        case '=':
        case '+':
          event.preventDefault();
          transformRef.current?.zoomIn(0.35, 120);
          break;
        case '-':
        case '_':
          event.preventDefault();
          transformRef.current?.zoomOut(0.35, 120);
          break;
        case '0':
          event.preventDefault();
          fitToView();
          break;
        case '1':
          event.preventDefault();
          setActualSize();
          break;
      }
    };

    const onKeyUp = (event: KeyboardEvent) => {
      if (event.key === ' ') setSpacePressed(false);
    };
    const clearSpace = () => setSpacePressed(false);

    container.addEventListener('keydown', onKeyDown);
    container.addEventListener('keyup', onKeyUp);
    window.addEventListener('keyup', onKeyUp);
    window.addEventListener('blur', clearSpace);
    return () => {
      container.removeEventListener('keydown', onKeyDown);
      container.removeEventListener('keyup', onKeyUp);
      window.removeEventListener('keyup', onKeyUp);
      window.removeEventListener('blur', clearSpace);
    };
  }, [
    clearOristudioCpSelection,
    cpToolState,
    editableCp,
    editableSelectionSize,
    fitToView,
    hasCreasePattern,
    setActualSize,
  ]);

  useEffect(() => {
    if (!editableCp) {
      setCpToolPoints([]);
      setCpToolPath([]);
      cpToolDragRef.current = null;
      setCpToolState(IDLE_ORISTUDIO_CP_TOOL_STATE);
    }
  }, [editableCp]);

  useEffect(() => {
    setCpMeasurementSlots(createEmptyCpMeasurementSlots());
  }, [editableCpHandle]);

  return (
    <section className="panel-shell cp-panel">
      <div className="panel-toolbar">
        <div className="panel-toolbar__group">
          <span className="panel-title">Crease Pattern</span>
        </div>
        {hasCreasePattern ? (
          <div className="cp-panel__mode">
            <span className="cp-panel__mode-label">Color by</span>
            <SegmentedControl
              aria-label="Choose how crease lines are colored"
              value={mode}
              onChange={setMode}
              options={[
                {
                  value: 'agrh',
                  label: 'Crease roles',
                  icon: <ScanLine size={13} />,
                  title: 'Color by axial, gusset, ridge, hinge, and pseudohinge roles',
                },
                {
                  value: 'mvf',
                  label: 'M/V assignment',
                  icon: <GitBranch size={13} />,
                  title: 'Color by mountain, valley, flat, and border folds',
                },
              ]}
            />
          </div>
        ) : (
          <span className="panel-toolbar__meta">{emptyStatusLabel}</span>
        )}
      </div>
      {sourceLabel && <div className="panel-subtitle">{sourceLabel}</div>}
      <div
        ref={containerRef}
        className={[
          'panel-body cp-panel__body',
          editableCp ? 'cp-panel__body--with-tools' : '',
        ].join(' ')}
        data-space-pan={spacePressed || undefined}
        tabIndex={-1}
        onPointerDown={(event) => {
          if (!isViewportInteractiveTarget(event.target)) containerRef.current?.focus();
        }}
      >
        {hasCreasePattern ? (
          <>
            {editableCp && (
              <CpToolRail
                activeActionId={cpToolState.activeActionId}
                activeLineColor={activeCpLineColor}
                editable={!!editableCp}
                onSelectAction={handleCpToolAction}
              />
            )}
            <div className="cp-panel__viewport">
              <TransformWrapper
                ref={transformRef}
                initialScale={1}
                minScale={0.05}
                maxScale={30}
                centerOnInit
                limitToBounds={false}
                wheel={{ step: 0.5, wheelDisabled: true }}
                panning={{
                  velocityDisabled: true,
                  wheelPanning: true,
                  allowMiddleClickPan: true,
                  allowLeftClickPan: spacePressed,
                }}
                pinch={{ step: 0.5 }}
                doubleClick={{ disabled: true }}
                onInit={(ref) => {
                  transformRef.current = ref;
                  requestAnimationFrame(() => fitLoadedCreasePatternRef.current(0));
                }}
                onTransformed={(_ref, state) => setZoomPercent(Math.round(state.scale * 100))}
              >
                <TransformComponent
                  wrapperStyle={{ width: '100%', height: '100%' }}
                  contentStyle={{ width: 'fit-content', height: 'fit-content' }}
                >
                  <svg
                    ref={svgRef}
                    className="cp-canvas"
                    viewBox={`0 0 ${CP_VIEWBOX_SIZE} ${CP_VIEWBOX_SIZE}`}
                    width={CP_VIEWBOX_SIZE}
                    height={CP_VIEWBOX_SIZE}
                    style={{ width: CP_VIEWBOX_SIZE, height: CP_VIEWBOX_SIZE }}
                    role="img"
                    aria-label="Crease pattern"
                    onPointerMove={handleEditablePointerMove}
                    onPointerUp={finishEditableDragPath}
                    onPointerCancel={cancelEditableDragPath}
                    onPointerLeave={clearEditablePointerStatus}
                    onPointerDownCapture={handleEditableToolPointerDown}
                    onPointerDown={(event) => {
                      if (event.target === event.currentTarget) clearSelectionOnBackgroundPointerDown(event);
                    }}
                  >
                    <rect
                      className="paper-shadow"
                      x={CP_PAPER_SHADOW_RECT.x}
                      y={CP_PAPER_SHADOW_RECT.y}
                      width={CP_PAPER_SHADOW_RECT.width}
                      height={CP_PAPER_SHADOW_RECT.height}
                      rx="6"
                    />
                    <rect
                      className="paper"
                      x={CP_PAPER_RECT.x}
                      y={CP_PAPER_RECT.y}
                      width={CP_PAPER_RECT.width}
                      height={CP_PAPER_RECT.height}
                      onPointerDown={clearSelectionOnBackgroundPointerDown}
                    />
                    {editableCp ? (
                      <EditableCreasePattern
                        bounds={editableCpBounds}
                        clearSelectionOnBackgroundPointerDown={clearSelectionOnBackgroundPointerDown}
                        document={editableCp}
                        gridLines={editableCpGridLines}
                        gridVisible={oristudioCpViewport.gridVisible}
                        mode={mode}
                        commandCandidatePoints={cpCommandPreview?.points ?? []}
                        commandPreviewCircles={renderedCommandPreviewCircles}
                        commandPreviewPoints={renderedCommandPreviewPoints}
                        commandPreviewSegments={renderedCommandPreviewSegments}
                        selection={oristudioCpSelection}
                        snapTarget={snapTarget}
                        spacePressed={spacePressed}
                        toggleCircle={handleEditableCircleClick}
                        toggleLine={handleEditableLineClick}
                        togglePoint={handleEditablePointClick}
                        toggleText={handleEditableTextClick}
                        toggleVertex={handleEditableVertexClick}
                        vertices={editableCpVertices}
                      />
                    ) : (
                      <GeneratedCreasePattern
                        clearSelectionOnBackgroundPointerDown={clearSelectionOnBackgroundPointerDown}
                        mode={mode}
                        project={project}
                        select={select}
                        selection={selection}
                        spacePressed={spacePressed}
                      />
                    )}
                  </svg>
                </TransformComponent>
              </TransformWrapper>
              <ViewportToolbar
                ariaLabel="Crease pattern viewport controls"
                zoomPercent={zoomPercent}
                zoomIn={() => transformRef.current?.zoomIn(0.35, 120)}
                zoomOut={() => transformRef.current?.zoomOut(0.35, 120)}
                fitToView={() => fitToView()}
                setActualSize={setActualSize}
                setZoomLevel={setZoomLevel}
              >
                {editableCp && (
                  <>
                    <ViewportToolbarSeparator />
                    <IconButton
                      size="sm"
                      variant="toolbar"
                      title="Grid"
                      isActive={oristudioCpViewport.gridVisible}
                      onClick={() =>
                        setOristudioCpViewportOption('gridVisible', !oristudioCpViewport.gridVisible)
                      }
                    >
                      <Grid2X2 size={14} />
                    </IconButton>
                    <IconButton
                      size="sm"
                      variant="toolbar"
                      title="Snap"
                      isActive={
                        oristudioCpViewport.snapToGrid ||
                        oristudioCpViewport.snapToVertices ||
                        oristudioCpViewport.snapToLines
                      }
                      onClick={() => {
                        const enabled =
                          oristudioCpViewport.snapToGrid ||
                          oristudioCpViewport.snapToVertices ||
                          oristudioCpViewport.snapToLines;
                        setOristudioCpViewportOption('snapToGrid', !enabled);
                        setOristudioCpViewportOption('snapToVertices', !enabled);
                        setOristudioCpViewportOption('snapToLines', !enabled);
                      }}
                    >
                      <Magnet size={14} />
                    </IconButton>
                  </>
                )}
              </ViewportToolbar>
              {editableCp && activeCpCommand && (
                <CpContextToolPanel
                  command={activeCpCommand}
                  options={cpToolOptions}
                  setOptions={setCpToolOptions}
                  activeLineColor={activeCpLineColor}
                  measurementSlots={cpMeasurementSlots}
                  onApply={
                    cpCommandRequiresContextApply(activeCpCommand)
                      ? handleApplyActiveContextCommand
                      : undefined
                  }
                />
              )}
              <div className="viewport-status-readout">
                <span>{formatZoom(zoomPercent / 100)}</span>
                {editableCp && <span>{cpToolState.prompt}</span>}
                {editableCp && <span>{cpLineTypeStatusLabel(activeCpLineColor)}</span>}
                {editableCp && editableCpSummary && (
                  <span>{editableCpSummary.line_segments} lines</span>
                )}
                {editableCp && cursorModelPoint && (
                  <span>
                    {formatNumber(cursorModelPoint.x, 2)}, {formatNumber(cursorModelPoint.y, 2)}
                  </span>
                )}
                {editableCp && snapTarget && <span>Snap {snapTarget.label}</span>}
                {editableCp && editableSelectionSize > 0 && (
                  <span>{editableSelectionSize} selected</span>
                )}
              </div>
            </div>
          </>
        ) : (
          <div className="cp-panel__empty">
            <span title={status === 'error' ? error?.message : undefined}>{emptyStatusLabel}</span>
            <NextDocumentAction />
          </div>
        )}
      </div>
    </section>
  );
}

interface EditableCreasePatternProps {
  bounds: ReturnType<typeof getEditableCpModelBounds>;
  clearSelectionOnBackgroundPointerDown: (event: PointerEvent<SVGElement>) => void;
  document: OristudioCpDocumentSnapshot;
  gridLines: ReturnType<typeof getCpGridLines>;
  gridVisible: boolean;
  mode: 'mvf' | 'agrh';
  commandCandidatePoints: Point[];
  commandPreviewCircles: OristudioCpCircle[];
  commandPreviewPoints: Point[];
  commandPreviewSegments: OristudioCpLineSegment[];
  selection: OristudioCpSelection;
  snapTarget: CpSnapTarget | null;
  spacePressed: boolean;
  toggleCircle: (id: number, additive?: boolean) => void;
  toggleLine: (id: number, additive?: boolean) => void;
  togglePoint: (id: number, additive?: boolean) => void;
  toggleText: (id: number, additive?: boolean) => void;
  toggleVertex: (id: string, additive?: boolean) => void;
  vertices: CpVertex[];
}

function EditableCreasePattern({
  bounds,
  clearSelectionOnBackgroundPointerDown,
  document,
  gridLines,
  gridVisible,
  mode,
  commandCandidatePoints,
  commandPreviewCircles,
  commandPreviewPoints,
  commandPreviewSegments,
  selection,
  snapTarget,
  spacePressed,
  toggleCircle,
  toggleLine,
  togglePoint,
  toggleText,
  toggleVertex,
  vertices,
}: EditableCreasePatternProps) {
  return (
    <>
      {gridVisible &&
        gridLines.map((line) => {
          const a = modelPointToCpSvg(line.a, bounds);
          const b = modelPointToCpSvg(line.b, bounds);
          return (
            <line
              key={line.id}
              className={['cp-grid-line', line.major ? 'cp-grid-line--major' : ''].join(' ')}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
            />
          );
        })}
      {document.crease_pattern.line_segments.map((line, index) => {
        const id = index + 1;
        const a = modelPointToCpSvg(line.a, bounds);
        const b = modelPointToCpSvg(line.b, bounds);
        return (
          <g key={id}>
            <line
              className="cp-line-hit-target"
              data-cp-line-hit-id={id}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
              aria-label={`Editable ${cpLineAssignmentLabel(line.color)} line ${id} hit target`}
              onClick={(event) => {
                if (spacePressed) return;
                event.stopPropagation();
                toggleLine(id, event.shiftKey || event.metaKey || event.ctrlKey);
              }}
            />
            <line
              className={[
                cpLineColorClass(line.color, mode),
                selection.lines.includes(id) ? 'crease--selected' : '',
              ].join(' ')}
              data-cp-line-id={id}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
              aria-label={`Editable ${cpLineAssignmentLabel(line.color)} line ${id}`}
              onClick={(event) => {
                if (spacePressed) return;
                event.stopPropagation();
                toggleLine(id, event.shiftKey || event.metaKey || event.ctrlKey);
              }}
            />
          </g>
        );
      })}
      {document.crease_pattern.points.map((point, index) => {
        const id = index + 1;
        const svgPoint = modelPointToCpSvg(point, bounds);
        return (
          <circle
            key={id}
            className={[
              'cp-point',
              selection.points.includes(id) ? 'cp-point--selected' : '',
            ].join(' ')}
            cx={svgPoint.x}
            cy={svgPoint.y}
            r="4"
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              togglePoint(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {document.crease_pattern.circles.map((circle, index) => {
        const id = index + 1;
        const center = modelPointToCpSvg({ x: circle.x, y: circle.y }, bounds);
        const radius =
          (circle.r / Math.max(bounds.spanX, bounds.spanY)) *
          Math.min(CP_PAPER_RECT.width, CP_PAPER_RECT.height);
        return (
          <circle
            key={id}
            className={[
              'cp-circle',
              selection.circles.includes(id) ? 'cp-circle--selected' : '',
            ].join(' ')}
            cx={center.x}
            cy={center.y}
            r={Math.max(1, radius)}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleCircle(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {commandPreviewCircles.map((circle, index) => {
        const center = modelPointToCpSvg({ x: circle.x, y: circle.y }, bounds);
        const radius =
          (circle.r / Math.max(bounds.spanX, bounds.spanY)) *
          Math.min(CP_PAPER_RECT.width, CP_PAPER_RECT.height);
        return (
          <circle
            key={`${index}-${circle.x}-${circle.y}-${circle.r}`}
            className="cp-command-preview"
            cx={center.x}
            cy={center.y}
            r={Math.max(1, radius)}
          />
        );
      })}
      {document.crease_pattern.texts.map((text, index) => {
        const id = index + 1;
        const position = modelPointToCpSvg(
          { x: textCoordinate(text.x), y: textCoordinate(text.y) },
          bounds
        );
        return (
          <text
            key={id}
            className={['cp-text', selection.texts.includes(id) ? 'cp-text--selected' : ''].join(
              ' '
            )}
            x={position.x}
            y={position.y}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleText(id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          >
            {text.text}
          </text>
        );
      })}
      {vertices.map((vertex) => {
        const svgPoint = modelPointToCpSvg(vertex.point, bounds);
        const selected = selection.vertices?.includes(vertex.id) ?? false;
        return (
          <circle
            key={vertex.id}
            className={['cp-vertex', selected ? 'cp-vertex--selected' : ''].join(' ')}
            data-cp-vertex-id={vertex.id}
            cx={svgPoint.x}
            cy={svgPoint.y}
            r="4.5"
            aria-label={`Editable vertex at ${formatNumber(vertex.point.x, 2)}, ${formatNumber(vertex.point.y, 2)}`}
            onClick={(event) => {
              if (spacePressed) return;
              event.stopPropagation();
              toggleVertex(vertex.id, event.shiftKey || event.metaKey || event.ctrlKey);
            }}
          />
        );
      })}
      {document.operation_frame?.active && (
        <polygon
          className="cp-operation-frame"
          points={document.operation_frame.points
            .map((point) => modelPointToCpSvg(point, bounds))
            .map((point) => `${point.x},${point.y}`)
          .join(' ')}
        />
      )}
      {commandPreviewSegments.map((segment, index) => {
        const a = modelPointToCpSvg(segment.a, bounds);
        const b = modelPointToCpSvg(segment.b, bounds);
        return (
          <line
            key={`${index}-${segment.a.x}-${segment.a.y}-${segment.b.x}-${segment.b.y}`}
            className={[
              cpLineColorClass(segment.color, mode),
              'cp-command-candidate',
            ].join(' ')}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
          />
        );
      })}
      {commandPreviewPoints.length > 1 && (
        <polyline
          className="cp-command-preview"
          points={commandPreviewPoints
            .map((point) => modelPointToCpSvg(point, bounds))
            .map((point) => `${point.x},${point.y}`)
            .join(' ')}
        />
      )}
      {commandCandidatePoints.map((point, index) => {
        const svgPoint = modelPointToCpSvg(point, bounds);
        return (
          <circle
            key={`${index}-${point.x}-${point.y}`}
            className="cp-command-candidate-point"
            cx={svgPoint.x}
            cy={svgPoint.y}
            r="4"
          />
        );
      })}
      {snapTarget && (
        <circle
          className="cp-snap-target"
          cx={modelPointToCpSvg(snapTarget.point, bounds).x}
          cy={modelPointToCpSvg(snapTarget.point, bounds).y}
          r="5"
        />
      )}
      <rect
        className="paper-border"
        x={CP_PAPER_RECT.x}
        y={CP_PAPER_RECT.y}
        width={CP_PAPER_RECT.width}
        height={CP_PAPER_RECT.height}
        onPointerDown={clearSelectionOnBackgroundPointerDown}
      />
    </>
  );
}

interface GeneratedCreasePatternProps {
  clearSelectionOnBackgroundPointerDown: (event: PointerEvent<SVGElement>) => void;
  mode: 'mvf' | 'agrh';
  project: TreeProject;
  select: (selection: Selection) => void;
  selection: Selection;
  spacePressed: boolean;
}

function GeneratedCreasePattern({
  clearSelectionOnBackgroundPointerDown,
  mode,
  project,
  select,
  selection,
  spacePressed,
}: GeneratedCreasePatternProps) {
  return (
    <>
      {project.facets.map((facet) => {
        const points = facet.vertices
          .map((point) => paperToSvg(point, CP_PAPER_RECT))
          .map((point) => `${point.x},${point.y}`)
          .join(' ');
        return (
          <polygon
            key={facet.id}
            className={[
              `facet facet--${facet.color}`,
              isFacetSelected(selection, facet.id) ? 'facet--selected' : '',
            ].join(' ')}
            points={points}
            onClick={(event) => {
              if (spacePressed) return;
              select(
                event.shiftKey || event.metaKey || event.ctrlKey
                  ? toggleFacetSelection(selection, facet.id)
                  : { kind: 'facet', id: facet.id }
              );
            }}
          />
        );
      })}
      {project.creases.map((crease) => {
        const a = paperToSvg(crease.vertices[0], CP_PAPER_RECT);
        const b = paperToSvg(crease.vertices[1], CP_PAPER_RECT);
        return (
          <line
            key={crease.id}
            className={[
              creaseClass(crease.fold, crease.kind, mode),
              isCreaseSelected(selection, crease.id) ? 'crease--selected' : '',
            ].join(' ')}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            onClick={(event) => {
              if (spacePressed) return;
              select(
                event.shiftKey || event.metaKey || event.ctrlKey
                  ? toggleCreaseSelection(selection, crease.id)
                  : { kind: 'crease', id: crease.id }
              );
            }}
          />
        );
      })}
      <rect
        className="paper-border"
        x={CP_PAPER_RECT.x}
        y={CP_PAPER_RECT.y}
        width={CP_PAPER_RECT.width}
        height={CP_PAPER_RECT.height}
        onPointerDown={clearSelectionOnBackgroundPointerDown}
      />
    </>
  );
}

function CpContextToolPanel({
  command,
  options,
  setOptions,
  activeLineColor,
  measurementSlots,
  onApply,
}: {
  command: OristudioCpCommandDefinition;
  options: OristudioCpToolOptions;
  setOptions: Dispatch<SetStateAction<OristudioCpToolOptions>>;
  activeLineColor: OristudioCpLineColor;
  measurementSlots: CpMeasurementSlots;
  onApply?: () => void;
}) {
  const [collapsed, setCollapsed] = useState(false);
  const groups = cpToolSettingGroupsForCommand(command);

  if (groups.length === 0) return null;

  return (
    <section
      className="cp-context-panel"
      aria-label="Crease pattern tool options"
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => event.stopPropagation()}
    >
      <button
        className="cp-context-panel__header"
        type="button"
        aria-expanded={!collapsed}
        onClick={() => setCollapsed((current) => !current)}
      >
        {collapsed ? <ChevronRight size={13} /> : <ChevronDown size={13} />}
        <span className="cp-context-panel__title">{command.label}</span>
        <span className="cp-context-panel__meta">
          {groups.length} {groups.length === 1 ? 'setting' : 'settings'}
        </span>
      </button>
      {!collapsed && (
        <div className="cp-context-panel__body">
          {groups.map((group) => (
            <CpContextToolGroup
              key={group}
              group={group}
              options={options}
              setOptions={setOptions}
              activeLineColor={activeLineColor}
              activeMeasurementSlot={cpMeasurementSlotForOperation(command.operationId)}
              measurementSlots={measurementSlots}
            />
          ))}
          {onApply && (
            <button className="cp-context-panel__apply" type="button" onClick={onApply}>
              Apply to selection
            </button>
          )}
        </div>
      )}
    </section>
  );
}

function CpContextToolGroup({
  group,
  options,
  setOptions,
  activeLineColor,
  activeMeasurementSlot,
  measurementSlots,
}: {
  group: OristudioCpToolSettingGroup;
  options: OristudioCpToolOptions;
  setOptions: Dispatch<SetStateAction<OristudioCpToolOptions>>;
  activeLineColor: OristudioCpLineColor;
  activeMeasurementSlot: CpMeasurementSlotId | null;
  measurementSlots: CpMeasurementSlots;
}) {
  if (group === 'line-color') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Line type</div>
        <div className="cp-context-panel__readout">{cpLineTypeStatusLabel(activeLineColor)}</div>
      </div>
    );
  }

  if (group === 'division-count') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Divide by count</div>
        <NumericToolOption
          label="Count"
          ariaLabel="Division count"
          min={1}
          max={256}
          step={1}
          value={options.divisionCount}
          onChange={(divisionCount) =>
            setOptions((current) => ({ ...current, divisionCount }))
          }
        />
      </div>
    );
  }

  if (group === 'division-ratio') {
    return <DivisionRatioOptions options={options} setOptions={setOptions} />;
  }

  if (group === 'angle-system') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Angle system</div>
        <NumericToolOption
          label="Divider"
          ariaLabel="Angle system divider"
          min={0}
          max={360}
          step={1}
          value={options.angleSystemDivider}
          onChange={(angleSystemDivider) =>
            setOptions((current) => ({ ...current, angleSystemDivider }))
          }
        />
        <div className="cp-context-panel__angle-grid">
          {ANGLE_FIELDS.map((field, index) => (
            <NumericToolOption
              key={field}
              label={field}
              ariaLabel={`Angle ${field}`}
              min={0}
              max={360}
              step={0.1}
              value={options.angleSystemAngles[index] ?? 0}
              onChange={(value) => updateAngleField(setOptions, index, value)}
            />
          ))}
        </div>
      </div>
    );
  }

  if (group === 'replace-line-type') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Replace line type</div>
        <SelectToolOption
          label="From"
          ariaLabel="Replace from line type"
          value={options.customFromLineType}
          options={ORISTUDIO_CP_CUSTOM_LINE_TYPE_OPTIONS}
          onChange={(customFromLineType) =>
            setOptions((current) => ({ ...current, customFromLineType }))
          }
        />
        <SelectToolOption
          label="To"
          ariaLabel="Replace to line type"
          value={options.customToLineType}
          options={ORISTUDIO_CP_REPLACE_TARGET_LINE_TYPE_OPTIONS}
          onChange={(customToLineType) =>
            setOptions((current) => ({ ...current, customToLineType }))
          }
        />
      </div>
    );
  }

  if (group === 'delete-line-type') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Delete line type</div>
        <SelectToolOption
          label="Filter"
          ariaLabel="Delete line type"
          value={options.customLineType}
          options={ORISTUDIO_CP_CUSTOM_LINE_TYPE_OPTIONS}
          onChange={(customLineType) =>
            setOptions((current) => ({ ...current, customLineType }))
          }
        />
      </div>
    );
  }

  if (group === 'fix-precision') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Fix inaccurate</div>
        <NumericToolOption
          label="Precision"
          ariaLabel="Fix precision"
          min={0}
          max={100}
          step={0.01}
          value={options.fixPrecision}
          onChange={(fixPrecision) => setOptions((current) => ({ ...current, fixPrecision }))}
        />
        <CheckboxToolOption
          label="BP"
          ariaLabel="Use BP fix targets"
          checked={options.fixPrecisionUseBp}
          onChange={(fixPrecisionUseBp) =>
            setOptions((current) => ({ ...current, fixPrecisionUseBp }))
          }
        />
        <CheckboxToolOption
          label="22.5"
          ariaLabel="Use 22.5 fix targets"
          checked={options.fixPrecisionUse22_5}
          onChange={(fixPrecisionUse22_5) =>
            setOptions((current) => ({ ...current, fixPrecisionUse22_5 }))
          }
        />
      </div>
    );
  }

  if (group === 'polygon-corners') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Regular polygon</div>
        <NumericToolOption
          label="Corners"
          ariaLabel="Polygon corners"
          min={3}
          max={256}
          step={1}
          value={options.polygonCorners}
          onChange={(polygonCorners) =>
            setOptions((current) => ({ ...current, polygonCorners }))
          }
        />
      </div>
    );
  }

  if (group === 'parallel-width') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Parallel width</div>
        <NumericToolOption
          label="Width"
          ariaLabel="Parallel width"
          min={0}
          max={9999}
          step={0.1}
          value={options.parallelWidth}
          onChange={(parallelWidth) =>
            setOptions((current) => ({ ...current, parallelWidth }))
          }
        />
      </div>
    );
  }

  if (group === 'candidate-choice') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Candidate</div>
        <CheckboxToolOption
          label="Auto nearest"
          ariaLabel="Use nearest candidate"
          checked={options.candidateIndex === null}
          onChange={(useNearest) =>
            setOptions((current) => ({
              ...current,
              candidateIndex: useNearest ? null : 0,
            }))
          }
        />
        <NumericToolOption
          label="Index"
          ariaLabel="Candidate index"
          min={1}
          max={256}
          step={1}
          value={(options.candidateIndex ?? 0) + 1}
          disabled={options.candidateIndex === null}
          onChange={(candidateIndex) =>
            setOptions((current) => ({
              ...current,
              candidateIndex: Math.max(0, candidateIndex - 1),
            }))
          }
        />
      </div>
    );
  }

  if (group === 'apply-lines') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Apply lines</div>
        <div className="cp-context-panel__readout">
          Seed lines and apply action will unlock with the generator port.
        </div>
      </div>
    );
  }

  if (group === 'measurement-readout') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Measurement</div>
        <div className="cp-context-panel__measurement-grid">
          {CP_MEASUREMENT_SLOT_ORDER.map((slot) => (
            <div
              key={slot}
              className="cp-context-panel__measurement-row"
              data-active={slot === activeMeasurementSlot || undefined}
              data-measurement-slot={slot}
            >
              <span>{CP_MEASUREMENT_SLOT_LABELS[slot]}</span>
              <span>{formatCpMeasurementValue(slot, measurementSlots[slot])}</span>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (group === 'custom-circle-color') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Circle color</div>
        <div
          className="cp-context-panel__color-swatch"
          style={{
            backgroundColor: `rgb(${options.customCircleColor.red}, ${options.customCircleColor.green}, ${options.customCircleColor.blue})`,
          }}
          aria-hidden="true"
        />
        <div className="cp-context-panel__angle-grid">
          {RGB_FIELDS.map((field) => (
            <NumericToolOption
              key={field.key}
              label={field.label}
              ariaLabel={field.ariaLabel}
              min={0}
              max={255}
              step={1}
              value={options.customCircleColor[field.key]}
              onChange={(value) => updateCustomCircleColor(setOptions, field.key, value)}
            />
          ))}
        </div>
      </div>
    );
  }

  if (group === 'line-select-help') {
    return (
      <div className="cp-context-panel__group">
        <div className="cp-context-panel__group-title">Line selection</div>
        <div className="cp-context-panel__readout">Drag across creases to apply this action.</div>
      </div>
    );
  }

  return null;
}

const RATIO_FIELDS: readonly {
  key: keyof OristudioCpRatioExpression;
  label: string;
  ariaLabel: string;
  min?: number;
  step: number;
}[] = [
  { key: 'a', label: 'A', ariaLabel: 'Ratio A', step: 0.1 },
  { key: 'b', label: 'B', ariaLabel: 'Ratio B', step: 0.1 },
  { key: 'c', label: 'C', ariaLabel: 'Ratio C', min: 0, step: 0.1 },
  { key: 'd', label: 'D', ariaLabel: 'Ratio D', step: 0.1 },
  { key: 'e', label: 'E', ariaLabel: 'Ratio E', step: 0.1 },
  { key: 'f', label: 'F', ariaLabel: 'Ratio F', min: 0, step: 0.1 },
];

function DivisionRatioOptions({
  options,
  setOptions,
}: {
  options: OristudioCpToolOptions;
  setOptions: Dispatch<SetStateAction<OristudioCpToolOptions>>;
}) {
  const initialHalves = ratioHalvesFromExpression(options.divisionRatio);
  const [leftDraft, setLeftDraft] = useState(() => formatOrieditaRatioHalf(initialHalves.left));
  const [rightDraft, setRightDraft] = useState(() => formatOrieditaRatioHalf(initialHalves.right));
  const ratio = evaluateOrieditaRatioExpression(options.divisionRatio);
  const leftInvalid = parseOrieditaRatioHalfInput(leftDraft) === null;
  const rightInvalid = parseOrieditaRatioHalfInput(rightDraft) === null;

  const applyRatioExpression = useCallback(
    (divisionRatio: OristudioCpRatioExpression) => {
      const halves = ratioHalvesFromExpression(divisionRatio);
      setLeftDraft(formatOrieditaRatioHalf(halves.left));
      setRightDraft(formatOrieditaRatioHalf(halves.right));
      setOptions((current) => ({ ...current, divisionRatio }));
    },
    [setOptions]
  );

  const updateSimpleHalf = useCallback(
    (side: 'left' | 'right', value: string) => {
      if (side === 'left') {
        setLeftDraft(value);
      } else {
        setRightDraft(value);
      }
      const parsed = parseOrieditaRatioHalfInput(value);
      if (!parsed) return;
      setOptions((current) => {
        const halves = ratioHalvesFromExpression(current.divisionRatio);
        return {
          ...current,
          divisionRatio: ratioExpressionFromHalves(
            side === 'left' ? parsed : halves.left,
            side === 'right' ? parsed : halves.right
          ),
        };
      });
    },
    [setOptions]
  );

  const updateExactField = useCallback(
    (field: keyof OristudioCpRatioExpression, value: number) => {
      const divisionRatio = {
        ...options.divisionRatio,
        [field]: value,
      };
      applyRatioExpression(divisionRatio);
    },
    [applyRatioExpression, options.divisionRatio]
  );

  return (
    <div className="cp-context-panel__group">
      <div className="cp-context-panel__group-title">Divide by ratio</div>
      <div className="cp-context-panel__ratio-simple">
        <TextToolOption
          label="Left"
          ariaLabel="Left segment ratio"
          value={leftDraft}
          invalid={leftInvalid}
          onChange={(value) => updateSimpleHalf('left', value)}
        />
        <TextToolOption
          label="Right"
          ariaLabel="Right segment ratio"
          value={rightDraft}
          invalid={rightInvalid}
          onChange={(value) => updateSimpleHalf('right', value)}
        />
      </div>
      <div className="cp-context-panel__preset-grid" aria-label="Ratio presets">
        {ORISTUDIO_CP_RATIO_PRESETS.map((preset) => (
          <button
            key={preset.label}
            type="button"
            className="cp-context-panel__preset"
            data-active={sameRatioExpression(options.divisionRatio, preset.expression) || undefined}
            aria-label={`Use ${preset.label} ratio`}
            onClick={() => applyRatioExpression(preset.expression)}
          >
            {preset.label}
          </button>
        ))}
      </div>
      <div className="cp-context-panel__readout">
        Computed ratio {formatOrieditaRatioNumber(ratio.ratioS)} :{' '}
        {formatOrieditaRatioNumber(ratio.ratioT)}
      </div>
      <details className="cp-context-panel__details">
        <summary>Exact form</summary>
        <div className="cp-context-panel__ratio-grid">
          {RATIO_FIELDS.map((field) => (
            <NumericToolOption
              key={field.key}
              label={field.label}
              ariaLabel={field.ariaLabel}
              min={field.min}
              max={999}
              step={field.step}
              value={options.divisionRatio[field.key]}
              onChange={(value) => updateExactField(field.key, value)}
            />
          ))}
        </div>
      </details>
    </div>
  );
}

function sameRatioExpression(
  left: OristudioCpRatioExpression,
  right: OristudioCpRatioExpression
): boolean {
  return RATIO_FIELDS.every((field) => left[field.key] === right[field.key]);
}

const ANGLE_FIELDS = ['A', 'B', 'C', 'D', 'E', 'F'] as const;

const RGB_FIELDS: readonly {
  key: keyof OristudioCpRgbColor;
  label: string;
  ariaLabel: string;
}[] = [
  { key: 'red', label: 'R', ariaLabel: 'Circle color red' },
  { key: 'green', label: 'G', ariaLabel: 'Circle color green' },
  { key: 'blue', label: 'B', ariaLabel: 'Circle color blue' },
];

function updateAngleField(
  setOptions: Dispatch<SetStateAction<OristudioCpToolOptions>>,
  index: number,
  value: number
) {
  setOptions((current) => {
    const angleSystemAngles = [...current.angleSystemAngles] as OristudioCpToolOptions['angleSystemAngles'];
    angleSystemAngles[index] = value;
    return {
      ...current,
      angleSystemAngles,
    };
  });
}

function updateCustomCircleColor(
  setOptions: Dispatch<SetStateAction<OristudioCpToolOptions>>,
  field: keyof OristudioCpRgbColor,
  value: number
) {
  setOptions((current) => ({
    ...current,
    customCircleColor: {
      ...current.customCircleColor,
      [field]: Math.round(value),
    },
  }));
}

function NumericToolOption({
  label,
  ariaLabel,
  min,
  max,
  step,
  value,
  disabled = false,
  onChange,
}: {
  label: string;
  ariaLabel: string;
  min?: number;
  max?: number;
  step: number;
  value: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <label className="cp-context-panel__field">
      <span>{label}</span>
      <input
        aria-label={ariaLabel}
        type="number"
        min={min}
        max={max}
        step={step}
        value={value}
        disabled={disabled}
        onChange={(event) => {
          const parsed = Number.parseFloat(event.currentTarget.value);
          if (!Number.isFinite(parsed)) return;
          onChange(clampToolNumber(parsed, min, max));
        }}
      />
    </label>
  );
}

function TextToolOption({
  label,
  ariaLabel,
  value,
  invalid,
  onChange,
}: {
  label: string;
  ariaLabel: string;
  value: string;
  invalid: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <label className="cp-context-panel__field">
      <span>{label}</span>
      <input
        aria-label={ariaLabel}
        type="text"
        value={value}
        aria-invalid={invalid}
        data-invalid={invalid || undefined}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </label>
  );
}

function SelectToolOption({
  label,
  ariaLabel,
  value,
  options,
  onChange,
}: {
  label: string;
  ariaLabel: string;
  value: OristudioCpCustomLineType;
  options: readonly { value: OristudioCpCustomLineType; label: string }[];
  onChange: (value: OristudioCpCustomLineType) => void;
}) {
  return (
    <label className="cp-context-panel__field">
      <span>{label}</span>
      <select
        aria-label={ariaLabel}
        value={value}
        onChange={(event) => onChange(event.currentTarget.value as OristudioCpCustomLineType)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function CheckboxToolOption({
  label,
  ariaLabel,
  checked,
  onChange,
}: {
  label: string;
  ariaLabel: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="cp-context-panel__checkbox">
      <input
        aria-label={ariaLabel}
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
      <span>{label}</span>
    </label>
  );
}

function clampToolNumber(value: number, min: number | undefined, max: number | undefined): number {
  const lowerBounded = min === undefined ? value : Math.max(min, value);
  return max === undefined ? lowerBounded : Math.min(max, lowerBounded);
}

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Crease pattern unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}
