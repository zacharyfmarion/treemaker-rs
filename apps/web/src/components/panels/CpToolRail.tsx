import {
  AlignJustify,
  BadgeAlert,
  BadgeCheck,
  BetweenHorizontalEnd,
  BetweenHorizontalStart,
  BoxSelect,
  ChartNoAxesCombined,
  ChartPie,
  Circle,
  CircleDashed,
  CircleDot,
  CircleDotDashed,
  CircleEllipsis,
  CirclePlus,
  CircleSlash,
  CircleX,
  Combine,
  Copy,
  CopyPlus,
  CornerDownLeft,
  Divide,
  Download,
  DraftingCompass,
  Eraser,
  FileCog,
  FileInput,
  FileOutput,
  FileSearch,
  FlipHorizontal,
  FoldHorizontal,
  GitBranch,
  Hand,
  Hexagon,
  Image,
  Lasso,
  LassoSelect,
  Layers,
  ListOrdered,
  ListPlus,
  ListRestart,
  Mountain,
  MousePointer2,
  MousePointerClick,
  Move,
  Network,
  Paintbrush,
  Palette,
  PenLine,
  PenTool,
  RefreshCw,
  Repeat,
  Repeat2,
  Replace,
  Ruler,
  RulerDimensionLine,
  Scan,
  ScanLine,
  ScanSearch,
  ShieldAlert,
  ShieldCheck,
  Shuffle,
  SkipForward,
  Sparkles,
  Split,
  SquareDashed,
  StretchHorizontal,
  TextCursorInput,
  Trash2,
  Unlink,
  VenetianMask,
  Waves,
  Wrench,
  type LucideIcon,
} from 'lucide-react';
import {
  ORISTUDIO_CP_ACTIONS,
  ORISTUDIO_CP_ACTION_GROUPS,
  cpActionsForGroup,
  type OristudioCpActionDefinition,
  type OristudioCpActionId,
} from '../../lib/oristudioCpActions';
import type { OristudioCpLineColor } from '../../engine/oristudioCpTypes';
import type { OristudioCpOperationId } from '../../lib/oristudioCpCommands';

interface CpToolRailProps {
  activeActionId: OristudioCpActionId | null;
  activeLineColor: OristudioCpLineColor;
  editable: boolean;
  onSelectAction: (action: OristudioCpActionDefinition) => void;
}

const LUCIDE_ICONS: Record<string, LucideIcon> = {
  AlignJustify,
  BadgeAlert,
  BadgeCheck,
  BetweenHorizontalEnd,
  BetweenHorizontalStart,
  BoxSelect,
  ChartNoAxesCombined,
  ChartPie,
  Circle,
  CircleDashed,
  CircleDot,
  CircleDotDashed,
  CircleEllipsis,
  CirclePlus,
  CircleSlash,
  CircleX,
  Combine,
  Copy,
  CopyPlus,
  CornerDownLeft,
  Divide,
  Download,
  DraftingCompass,
  Eraser,
  FileCog,
  FileInput,
  FileOutput,
  FileSearch,
  FlipHorizontal,
  FoldHorizontal,
  GitBranch,
  Hand,
  Hexagon,
  Image,
  Lasso,
  LassoSelect,
  Layers,
  ListOrdered,
  ListPlus,
  ListRestart,
  Mountain,
  MousePointer2,
  MousePointerClick,
  Move,
  Network,
  Paintbrush,
  Palette,
  PenLine,
  PenTool,
  RefreshCw,
  Repeat,
  Repeat2,
  Replace,
  Ruler,
  RulerDimensionLine,
  Scan,
  ScanLine,
  ScanSearch,
  ShieldAlert,
  ShieldCheck,
  Shuffle,
  SkipForward,
  Sparkles,
  Split,
  SquareDashed,
  StretchHorizontal,
  TextCursorInput,
  Trash2,
  Unlink,
  VenetianMask,
  Waves,
  Wrench,
};

const ICON_ALIASES: Record<string, string> = {
  angle: 'ChartNoAxesCombined',
  compass: 'DraftingCompass',
  divide: 'Divide',
  frame: 'Scan',
  mountain: 'Mountain',
  origami: 'Layers',
  split: 'Split',
};

const CP_TOOL_ICON_BY_OPERATION = Object.fromEntries(
  ORISTUDIO_CP_ACTIONS.filter((action) => action.kind === 'command').map((action) => [
    action.operationId,
    commandIcon(action.icon),
  ])
) as Partial<Record<OristudioCpOperationId, LucideIcon>>;

const CP_TOOL_ICON_BY_ACTION = Object.fromEntries(
  ORISTUDIO_CP_ACTIONS.map((action) => [action.id, commandIcon(action.icon)])
) as Partial<Record<OristudioCpActionId, LucideIcon>>;

const ORIEDITA_GLYPH_BY_ACTION_KEY: Record<string, string> = {
  drawCreaseFreeAction: '\uE000',
  h_senbun_nyuryokuAction: '\uE000',
  drawCreaseRestrictedAction: '\uE001',
  voronoiAction: '\uE002',
  makeFlatFoldableAction: '\uE003',
  lengthenCreaseAction: '\uE004',
  lengthenCrease2Action: '\uE005',
  angleBisectorAction: '\uE006',
  rabbitEarAction: '\uE007',
  perpendicularDrawAction: '\uE008',
  symmetricDrawAction: '\uE009',
  continuousSymmetricDrawAction: '\uE00A',
  parallelDrawAction: '\uE00B',
  setParallelDrawWidthAction: '\uE00C',
  foldableLineDrawAction: '\uE00D',
  foldableLinePlusGridInputAction: '\uE00D',
  fishBoneDrawAction: '\uE00F',
  doubleSymmetricDrawAction: '\uE010',
  senbun_b_nyuryokuAction: '\uE011',
  reflectAction: '\uE012',
  lineSegmentDeleteAction: '\uE013',
  del_l_typeAction: '\uE013',
  edgeLineSegmentDeleteAction: '\uE014',
  auxLiveLineSegmentDeleteAction: '\uE015',
  replace_lineAction: '\uE017',
  toMountainAction: '\uE017',
  toValleyAction: '\uE018',
  toEdgeAction: '\uE019',
  toAuxAction: '\uE01A',
  senbun_henkan2Action: '\uE01B',
  senbun_henkanAction: '\uE01C',
  senbun_yoke_henkanAction: '\uE01C',
  in_L_col_changeAction: '\uE01D',
  on_L_col_changeAction: '\uE01E',
  vertexAddAction: '\uE01F',
  vertexDeleteAction: '\uE020',
  v_del_ccAction: '\uE021',
  v_del_allAction: '\uE022',
  v_del_all_ccAction: '\uE023',
  drawTwoColoredCpAction: '\uE029',
  suitei_02Action: '\uE02A',
  suitei_03Action: '\uE02B',
  coloredXRayIncreaseAction: '\uE02C',
  ck4_colorIncreaseAction: '\uE02C',
  coloredXRayDecreaseAction: '\uE02D',
  ck4_colorDecreaseAction: '\uE02D',
  undoAction: '\uE02E',
  redoAction: '\uE02F',
  foldAction: '\uE035',
  oriagari_sousaAction: '\uE036',
  oriagari_sousa_2Action: '\uE037',
  foldedFigureFlipAction: '\uE038',
  haltAction: '\uE03E',
  foldedFigureTrashAction: '\uE03F',
  resetAction: '\uE040',
  operationFrameSelectAction: '\uE041',
  mouseSettingsAction: '\uE042',
  drawLineSegmentInternalDivisionRatioAction: '\uE044',
  moveCreasePatternAction: '\uE045',
  foldedFigureMoveAction: '\uE045',
  rotateClockwiseAction: '\uE046',
  rotateAnticlockwiseAction: '\uE047',
  deg1Action: '\uE04A',
  deg2Action: '\uE04C',
  deg3Action: '\uE04D',
  deg4Action: '\uE04E',
  regularPolygonAction: '\uE04F',
  circleDrawFreeAction: '\uE050',
  circleDrawAction: '\uE051',
  circleDrawSeparateAction: '\uE052',
  circleDrawConcentricSelectAction: '\uE053',
  circleDrawConcentricAction: '\uE054',
  circleDrawTwoConcentricAction: '\uE055',
  circleDrawTangentLineAction: '\uE056',
  circleDrawThreePointAction: '\uE057',
  circleDrawInvertedAction: '\uE058',
  sen_tokutyuu_color_henkouAction: '\uE059',
  selectAction: '\uE05D',
  unselectAction: '\uE05F',
  moveAction: '\uE061',
  move2p2pAction: '\uE062',
  copyAction: '\uE063',
  copy2p2pAction: '\uE064',
  deleteSelectedLineSegmentAction: '\uE065',
  angleSystemAIncreaseAction: '\uE066',
  angleSystemADecreaseAction: '\uE067',
  select_lXAction: '\uE069',
  unselect_lXAction: '\uE06A',
  del_lAction: '\uE06B',
  del_l_XAction: '\uE06C',
  select_polygonAction: '\uE06D',
  unselect_polygonAction: '\uE06E',
  foldedFigureToggleShadowAction: '\uE070',
  addColorConstraintAction: '\uE071',
  scaleAction: '\uE074',
  duplicateFoldedModelAction: '\uE075',
  axiom5Action: '\uE076',
  axiom7Action: '\uE078',
  selectLassoAction: '\uE07A',
  unselectLassoAction: '\uE07B',
  l1Action: '\uE07C',
  l2Action: '\uE07C',
  a1Action: '\uE07D',
  a2Action: '\uE07D',
  a3Action: '\uE07D',
  fixInaccurateAction: '\uE089',
  drawBlintzAction: '\uE090',
  drawFishBaseAction: '\uE091',
  drawDoveBaseAction: '\uE092',
  drawBirdBaseAction: '\uE093',
  drawFrogBaseAction: '\uE094',
};

const ORIEDITA_ACTION_KEY_BY_OPERATION: Partial<Record<OristudioCpOperationId, string>> = {
  DrawCreaseFree: 'drawCreaseFreeAction',
  MoveCreasePattern: 'moveCreasePatternAction',
  LineSegmentDelete: 'lineSegmentDeleteAction',
  ChangeCreaseType: 'senbun_henkanAction',
  LengthenCrease: 'lengthenCreaseAction',
  SquareBisector: 'angleBisectorAction',
  Inward: 'rabbitEarAction',
  PerpendicularDraw: 'perpendicularDrawAction',
  SymmetricDraw: 'symmetricDrawAction',
  DrawCreaseRestricted: 'drawCreaseRestrictedAction',
  DrawCreaseSymmetric: 'symmetricDrawAction',
  DrawCreaseAngleRestricted: 'deg1Action',
  DrawPoint: 'vertexAddAction',
  DeletePoint: 'vertexDeleteAction',
  AngleSystem: 'deg3Action',
  DrawCreaseAngleRestricted3: 'deg4Action',
  CreaseSelect: 'selectAction',
  CreaseUnselect: 'unselectAction',
  CreaseMove: 'moveAction',
  CreaseCopy: 'copyAction',
  CreaseMakeMountain: 'toMountainAction',
  CreaseMakeValley: 'toValleyAction',
  CreaseMakeEdge: 'toEdgeAction',
  LineSegmentDivision: 'senbun_b_nyuryokuAction',
  LineSegmentRatioSet: 'drawLineSegmentInternalDivisionRatioAction',
  PolygonSetNoCorners: 'regularPolygonAction',
  CreaseAdvanceType: 'senbun_henkanAction',
  CreaseMove4p: 'move2p2pAction',
  CreaseCopy4p: 'copy2p2pAction',
  FishBoneDraw: 'fishBoneDrawAction',
  CreaseMakeMv: 'in_L_col_changeAction',
  DoubleSymmetricDraw: 'doubleSymmetricDrawAction',
  CreasesAlternateMv: 'on_L_col_changeAction',
  DrawCreaseAngleRestricted5: 'deg2Action',
  VertexMakeAngularlyFlatFoldable: 'makeFlatFoldableAction',
  FoldableLineInput: 'foldableLinePlusGridInputAction',
  ParallelDraw: 'parallelDrawAction',
  VertexDeleteOnCrease: 'v_del_ccAction',
  CircleDraw: 'circleDrawAction',
  CircleDrawThreePoint: 'circleDrawThreePointAction',
  CircleDrawSeparate: 'circleDrawSeparateAction',
  CircleDrawTangentLine: 'circleDrawTangentLineAction',
  CircleDrawInverted: 'circleDrawInvertedAction',
  CircleDrawFree: 'circleDrawFreeAction',
  CircleDrawConcentric: 'circleDrawConcentricAction',
  CircleDrawConcentricSelect: 'circleDrawConcentricSelectAction',
  CircleDrawConcentricTwoCircleSelect: 'circleDrawTwoConcentricAction',
  ParallelDrawWidth: 'setParallelDrawWidthAction',
  ContinuousSymmetricDraw: 'continuousSymmetricDrawAction',
  DisplayLengthBetweenPoints1: 'l1Action',
  DisplayLengthBetweenPoints2: 'l2Action',
  DisplayAngleBetweenThreePoints1: 'a1Action',
  DisplayAngleBetweenThreePoints2: 'a2Action',
  DisplayAngleBetweenThreePoints3: 'a3Action',
  CreaseToggleMv: 'senbun_henkan2Action',
  CircleChangeColor: 'sen_tokutyuu_color_henkouAction',
  CreaseMakeAux: 'toAuxAction',
  OperationFrameCreate: 'operationFrameSelectAction',
  VoronoiCreate: 'voronoiAction',
  CreaseDeleteOverlapping: 'del_lAction',
  CreaseDeleteIntersecting: 'del_l_XAction',
  SelectPolygon: 'select_polygonAction',
  UnselectPolygon: 'unselect_polygonAction',
  SelectLineIntersecting: 'select_lXAction',
  UnselectLineIntersecting: 'unselect_lXAction',
  LengthenCreaseSameColor: 'lengthenCrease2Action',
  FoldableLineDraw: 'foldableLineDrawAction',
  ReplaceLineTypeSelect: 'replace_lineAction',
  DeleteLineTypeSelect: 'del_l_typeAction',
  SelectLasso: 'selectLassoAction',
  UnselectLasso: 'unselectLassoAction',
  DrawBlintz: 'drawBlintzAction',
  DrawFishBase: 'drawFishBaseAction',
  DrawDoveBase: 'drawDoveBaseAction',
  DrawBirdBase: 'drawBirdBaseAction',
  DrawFrogBase: 'drawFrogBaseAction',
  Axiom5: 'axiom5Action',
  Axiom7: 'axiom7Action',
  FixInaccurate: 'fixInaccurateAction',
  Fold: 'foldAction',
  DuplicateFoldedModel: 'duplicateFoldedModelAction',
  TwoColoredCp: 'drawTwoColoredCpAction',
};

export function CpToolRail({
  activeActionId,
  activeLineColor,
  editable,
  onSelectAction,
}: CpToolRailProps) {
  return (
    <aside className="cp-tool-rail" aria-label="Crease pattern tools">
      <div className="cp-tool-rail__groups">
        {ORISTUDIO_CP_ACTION_GROUPS.map((group) => {
          const actions = cpActionsForGroup(group.id).filter(
            (action) =>
              action.placement === 'left-rail' ||
              action.placement === 'left-rail-overflow'
          );
          if (actions.length === 0) return null;

          return (
            <section key={group.id} className="cp-tool-rail__group" aria-label={group.label}>
              <div className="cp-tool-rail__group-label">{group.railLabel}</div>
              <div className="cp-tool-rail__buttons">
                {actions.map((action) => (
                  <CpToolButton
                    key={action.id}
                    action={action}
                    editable={editable}
                    isActive={
                      action.kind === 'line-type'
                        ? activeLineColor === action.lineColor
                        : activeActionId === action.id
                    }
                    onSelectAction={onSelectAction}
                  />
                ))}
              </div>
            </section>
          );
        })}
      </div>
    </aside>
  );
}

function CpToolButton({
  action,
  editable,
  isActive,
  onSelectAction,
}: {
  action: OristudioCpActionDefinition;
  editable: boolean;
  isActive: boolean;
  onSelectAction: (action: OristudioCpActionDefinition) => void;
}) {
  const Icon =
    action.kind === 'command'
      ? (CP_TOOL_ICON_BY_OPERATION[action.operationId] ?? CircleDashed)
      : (CP_TOOL_ICON_BY_ACTION[action.id] ?? CircleDashed);
  const glyph = orieditaGlyphForAction(action);
  const available = editable && action.uiStatus === 'ready';
  const statusLabel = commandStatusLabel(action, editable);

  return (
    <button
      type="button"
      className="cp-tool-rail__button"
      aria-label={action.label}
      aria-disabled={!available}
      data-active={isActive || undefined}
      data-action-kind={action.kind}
      data-line-color={action.kind === 'line-type' ? action.lineColor : undefined}
      data-ui-status={action.uiStatus}
      title={`${action.label} - ${statusLabel}`}
      onClick={() => onSelectAction(action)}
    >
      {action.railLabel ? (
        <span className="cp-tool-rail__button-label" aria-hidden="true">
          {action.railLabel}
        </span>
      ) : glyph ? (
        <span className="cp-tool-rail__glyph" aria-hidden="true">
          {glyph}
        </span>
      ) : (
        <Icon size={15} aria-hidden="true" />
      )}
      <span className="cp-tool-rail__status-dot" aria-hidden="true" />
    </button>
  );
}

function orieditaGlyphForAction(action: OristudioCpActionDefinition): string | undefined {
  const directGlyph = ORIEDITA_GLYPH_BY_ACTION_KEY[action.upstreamAction];
  if (directGlyph) return directGlyph;

  if (action.kind !== 'command') return undefined;
  const actionKey = ORIEDITA_ACTION_KEY_BY_OPERATION[action.operationId];
  return actionKey ? ORIEDITA_GLYPH_BY_ACTION_KEY[actionKey] : undefined;
}

function commandIcon(icon: string): LucideIcon {
  const aliased = ICON_ALIASES[icon];
  const pascal = aliased ?? icon.split('-').map(capitalize).join('');
  return LUCIDE_ICONS[pascal] ?? CircleDashed;
}

function capitalize(value: string): string {
  return value.length === 0 ? value : `${value[0].toUpperCase()}${value.slice(1)}`;
}

function commandStatusLabel(action: OristudioCpActionDefinition, editable: boolean): string {
  if (!editable) return 'Open an editable crease pattern first';
  if (action.uiStatus === 'ready') return action.tooltip;
  return action.disabledReason;
}
