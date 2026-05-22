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
  ORISTUDIO_CP_COMMANDS,
  ORISTUDIO_CP_COMMAND_GROUPS,
  cpCommandsForGroup,
  type OristudioCpCommandDefinition,
  type OristudioCpOperationId,
} from '../../lib/oristudioCpCommands';

interface CpToolRailProps {
  activeOperationId: OristudioCpOperationId | null;
  editable: boolean;
  onSelectCommand: (command: OristudioCpCommandDefinition) => void;
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
  ORISTUDIO_CP_COMMANDS.map((command) => [command.operationId, commandIcon(command.icon)])
) as Partial<Record<OristudioCpOperationId, LucideIcon>>;

export function CpToolRail({ activeOperationId, editable, onSelectCommand }: CpToolRailProps) {
  return (
    <aside className="cp-tool-rail" aria-label="Crease pattern tools">
      <div className="cp-tool-rail__header">Tools</div>
      <div className="cp-tool-rail__groups">
        {ORISTUDIO_CP_COMMAND_GROUPS.map((group) => {
          const commands = cpCommandsForGroup(group.id).filter(
            (command) =>
              command.placement === 'left-rail' ||
              command.placement === 'left-rail-overflow'
          );
          if (commands.length === 0) return null;

          return (
            <section key={group.id} className="cp-tool-rail__group" aria-label={group.label}>
              <div className="cp-tool-rail__group-label">{group.railLabel}</div>
              <div className="cp-tool-rail__buttons">
                {commands.map((command) => (
                  <CpToolButton
                    key={command.id}
                    command={command}
                    editable={editable}
                    isActive={activeOperationId === command.operationId}
                    onSelectCommand={onSelectCommand}
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
  command,
  editable,
  isActive,
  onSelectCommand,
}: {
  command: OristudioCpCommandDefinition;
  editable: boolean;
  isActive: boolean;
  onSelectCommand: (command: OristudioCpCommandDefinition) => void;
}) {
  const Icon = CP_TOOL_ICON_BY_OPERATION[command.operationId] ?? CircleDashed;
  const available = editable && command.uiStatus === 'ready';
  const statusLabel = commandStatusLabel(command, editable);

  return (
    <button
      type="button"
      className="cp-tool-rail__button"
      aria-label={command.label}
      aria-disabled={!available}
      data-active={isActive || undefined}
      data-ui-status={command.uiStatus}
      title={`${command.label} - ${statusLabel}`}
      onClick={() => onSelectCommand(command)}
    >
      <Icon size={15} aria-hidden="true" />
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

function commandStatusLabel(command: OristudioCpCommandDefinition, editable: boolean): string {
  if (!editable) return 'Open an editable crease pattern first';
  if (command.uiStatus === 'ready') return command.tooltip;
  return command.disabledReason;
}
