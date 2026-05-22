import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent,
} from 'react';
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchRef } from 'react-zoom-pan-pinch';
import { GitBranch, Grid2X2, Magnet, ScanLine } from 'lucide-react';
import type {
  OristudioCpCommandPayload,
  OristudioCpCommandPreview,
  OristudioCpDocumentSnapshot,
  OristudioCpLineColor,
  OristudioCpLineSegment,
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
  zoomScale: number
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
    operationId === 'Axiom7'
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
    payload.angle_system_divider = 4;
  }

  if (operationId === 'LineSegmentDivision') {
    payload.division_count = 2;
  }

  if (operationId === 'LineSegmentRatioSet') {
    payload.ratio_s = 1;
    payload.ratio_t = 1;
  }

  if (operationId === 'ReplaceLineTypeSelect') {
    payload.custom_from_line_type = 'Any';
    payload.custom_to_line_type = 'Edge';
  }

  if (operationId === 'DeleteLineTypeSelect') {
    payload.custom_line_type = 'Any';
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

function isCpLineEventTarget(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest('[data-cp-line-id], [data-cp-line-hit-id]') !== null
  );
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
  const [cpToolPoints, setCpToolPoints] = useState<Point[]>([]);
  const [cpToolPath, setCpToolPath] = useState<Point[]>([]);
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
        zoomPercent / 100
      ),
      ...payload,
    }),
    [activeCpLineColor, editableCpBounds, editableCpGridWidth, zoomPercent]
  );

  useEffect(() => {
    if (
      !editableCp ||
      !activeCpCommand ||
      activeCpCommand.uiStatus !== 'ready' ||
      cpToolState.phase !== 'active' ||
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

function shortStatus(message: string): string {
  const trimmed = message.trim();
  if (!trimmed) return 'Crease pattern unavailable';
  const sentence = trimmed.split(/[.;]\s+/u)[0] ?? trimmed;
  return sentence.length > 54 ? `${sentence.slice(0, 51)}...` : sentence;
}
