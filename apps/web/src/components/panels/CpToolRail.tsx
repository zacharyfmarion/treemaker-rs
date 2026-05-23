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
      ) : (
        <Icon size={15} aria-hidden="true" />
      )}
      <span className="cp-tool-rail__status-dot" aria-hidden="true" />
    </button>
  );
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
