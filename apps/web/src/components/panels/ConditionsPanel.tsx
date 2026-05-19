import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { ChevronRight, FlipHorizontal2, Link2, LockKeyhole, Plus, Ruler, Trash2 } from 'lucide-react';
import type { ConditionKind } from '../../engine/types';
import { conditionDetail, conditionTitle } from '../../lib/conditionLabels';
import {
  SYMMETRY_PRESET_LABELS,
  nextSymmetryOption,
  paperCenter,
  symmetryOptionForPreset,
  symmetrySelectValueForState,
  type SymmetryPreset,
  type SymmetrySelectValue,
} from '../../lib/symmetryPresets';
import {
  isConditionSelected,
  selectedEdgeIds,
  selectedNodeIds,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { IconButton } from '../ui/IconButton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/Select';

export function ConditionsPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const selection = useWorkspaceStore((state) => state.selection);
  const select = useWorkspaceStore((state) => state.select);
  const updatePaper = useWorkspaceStore((state) => state.updatePaper);
  const setSymmetry = useWorkspaceStore((state) => state.setSymmetry);
  const addCondition = useWorkspaceStore((state) => state.addCondition);
  const deleteCondition = useWorkspaceStore((state) => state.deleteCondition);
  const clearConditions = useWorkspaceStore((state) => state.clearConditions);
  const nodeIds = selectedNodeIds(selection);
  const edgeIds = selectedEdgeIds(selection);
  const selectedNode = nodeIds.length === 1 ? project.nodes.find((node) => node.id === nodeIds[0]) : null;
  const selectedEdge = edgeIds.length === 1 ? project.edges.find((edge) => edge.id === edgeIds[0]) : null;
  const selectedPath =
    selection.kind === 'path' ? project.paths.find((path) => path.id === selection.id) : null;
  const [angle, setAngle] = useState(0);
  const [quant, setQuant] = useState(4);
  const [quantOffset, setQuantOffset] = useState(0);
  const [symmetryAdvancedOpen, setSymmetryAdvancedOpen] = useState(false);
  const [symmetryModeOverride, setSymmetryModeOverride] = useState<SymmetrySelectValue | null>(null);
  const projectLoadId = useWorkspaceStore((state) => state.projectLoadId);
  const inferredSymmetryMode = symmetrySelectValueForState({
    hasSymmetry: project.hasSymmetry,
    symAngle: project.paper.symAngle,
    symLoc: project.paper.symLoc,
    paperWidth: project.paper.width,
    paperHeight: project.paper.height,
  });
  const symmetryMode = project.hasSymmetry ? (symmetryModeOverride ?? inferredSymmetryMode) : 'none';
  const presetSymmetryMode = symmetryMode === 'book' || symmetryMode === 'diagonal' ? symmetryMode : null;
  const activePresetOption = presetSymmetryMode
    ? symmetryOptionForPreset(presetSymmetryMode, project.paper.symAngle)
    : null;
  const nextSymmetryPresetOption = activePresetOption ? nextSymmetryOption(activePresetOption) : null;
  const symmetryPresetCenter = paperCenter(project.paper.width, project.paper.height);

  useEffect(() => {
    setSymmetryModeOverride(null);
  }, [projectLoadId]);

  const selectedLeafNodeIds = useMemo(
    () =>
      nodeIds.filter((id) => {
        const node = project.nodes.find((candidate) => candidate.id === id);
        return node?.isLeaf;
      }),
    [nodeIds, project.nodes]
  );

  const add = (kind: ConditionKind) => {
    void addCondition(kind);
  };

  const applySymmetryPreset = (preset: SymmetryPreset) => {
    const option = symmetryOptionForPreset(preset, project.paper.symAngle);
    setSymmetryModeOverride(preset);
    void setSymmetry({
      hasSymmetry: true,
      symAngle: option.angle,
      symLoc: symmetryPresetCenter,
    });
  };

  const setSymmetryMode = (value: SymmetrySelectValue) => {
    setSymmetryModeOverride(value);
    if (value === 'none') {
      void setSymmetry({ hasSymmetry: false });
      return;
    }
    if (value === 'custom') {
      void setSymmetry({ hasSymmetry: true });
      return;
    }
    applySymmetryPreset(value);
  };

  const flipSymmetryPreset = () => {
    if (!nextSymmetryPresetOption || !presetSymmetryMode) return;
    setSymmetryModeOverride(presetSymmetryMode);
    void setSymmetry({
      hasSymmetry: true,
      symAngle: nextSymmetryPresetOption.angle,
      symLoc: symmetryPresetCenter,
    });
  };

  const updateCustomSymmetry = (update: Parameters<typeof setSymmetry>[0]) => {
    setSymmetryModeOverride('custom');
    void setSymmetry({ hasSymmetry: true, ...update });
  };

  return (
    <section className="panel-shell conditions-panel">
      <div className="panel-toolbar">
        <span className="panel-title">Conditions</span>
        <button
          className="toolbar-text-button"
          type="button"
          disabled={project.conditions.length === 0}
          onClick={() => void clearConditions()}
        >
          Clear
        </button>
      </div>
      <div className="panel-body conditions-panel__body">
        <section className="condition-section">
          <div className="condition-section__title">Paper</div>
          <NumberControl
            label="Width"
            value={project.paper.width}
            min={0.1}
            step={0.05}
            onCommit={(width) => void updatePaper({ width })}
          />
          <NumberControl
            label="Height"
            value={project.paper.height}
            min={0.1}
            step={0.05}
            onCommit={(height) => void updatePaper({ height })}
          />
        </section>

        <section className="condition-section">
          <div className="condition-section__title">Symmetry</div>
          <div className="condition-control-row">
            <span>Type</span>
            <div className="condition-control-row__field">
              <div className="symmetry-preset-controls">
                <Select value={symmetryMode} onValueChange={(value) => setSymmetryMode(value as SymmetrySelectValue)}>
                  <SelectTrigger aria-label="Symmetry type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">None</SelectItem>
                    {Object.entries(SYMMETRY_PRESET_LABELS).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                    <SelectItem value="custom">Custom</SelectItem>
                  </SelectContent>
                </Select>
                <IconButton
                  size="sm"
                  variant="toolbar"
                  title={nextSymmetryPresetOption ? `Flip to ${nextSymmetryPresetOption.label}` : 'Choose Book or Diagonal to flip'}
                  aria-label={
                    nextSymmetryPresetOption
                      ? `Flip symmetry to ${nextSymmetryPresetOption.label}`
                      : 'Choose Book or Diagonal to flip symmetry'
                  }
                  disabled={!nextSymmetryPresetOption}
                  onClick={flipSymmetryPreset}
                >
                  <FlipHorizontal2 size={14} />
                </IconButton>
              </div>
              {nextSymmetryPresetOption && (
                <span className="symmetry-preset-label">{activePresetOption?.label}</span>
              )}
            </div>
          </div>
          <div className="condition-advanced">
            <button
              className="condition-disclosure"
              type="button"
              data-open={symmetryAdvancedOpen || undefined}
              aria-expanded={symmetryAdvancedOpen}
              onClick={() => setSymmetryAdvancedOpen((open) => !open)}
            >
              <ChevronRight size={13} />
              <span>Advanced symmetry options</span>
            </button>
            {symmetryAdvancedOpen && (
              <div className="condition-advanced__body">
                <NumberControl
                  label="Angle"
                  value={project.paper.symAngle}
                  step={1}
                  onCommit={(symAngle) => updateCustomSymmetry({ symAngle })}
                />
                <NumberControl
                  label="X"
                  value={project.paper.symLoc.x}
                  min={0}
                  max={project.paper.width}
                  step={0.01}
                  onCommit={(x) => updateCustomSymmetry({ symLoc: { ...project.paper.symLoc, x } })}
                />
                <NumberControl
                  label="Y"
                  value={project.paper.symLoc.y}
                  min={0}
                  max={project.paper.height}
                  step={0.01}
                  onCommit={(y) => updateCustomSymmetry({ symLoc: { ...project.paper.symLoc, y } })}
                />
              </div>
            )}
          </div>
        </section>

        <section className="condition-section">
          <div className="condition-section__title">Add From Selection</div>
          <div className="condition-actions">
            <ConditionAction
              icon={<LockKeyhole size={14} />}
              label="Fix node"
              disabled={!selectedNode?.isLeaf}
              onClick={() =>
                selectedNode &&
                add({
                  type: 'node_fixed',
                  node: selectedNode.id,
                  x_fixed: true,
                  y_fixed: true,
                  x_fix_value: selectedNode.loc.x,
                  y_fix_value: selectedNode.loc.y,
                })
              }
            />
            <ConditionAction
              icon={<Ruler size={14} />}
              label="Node on edge"
              disabled={!selectedNode?.isLeaf}
              onClick={() => selectedNode && add({ type: 'node_on_edge', node: selectedNode.id })}
            />
            <ConditionAction
              icon={<Ruler size={14} />}
              label="Node on corner"
              disabled={!selectedNode?.isLeaf}
              onClick={() => selectedNode && add({ type: 'node_on_corner', node: selectedNode.id })}
            />
            <ConditionAction
              icon={<Link2 size={14} />}
              label="Node on symmetry"
              disabled={!selectedNode?.isLeaf || !project.hasSymmetry}
              onClick={() => selectedNode && add({ type: 'node_symmetric', node: selectedNode.id })}
            />
            <ConditionAction
              icon={<Link2 size={14} />}
              label="Pair nodes"
              disabled={selectedLeafNodeIds.length !== 2}
              onClick={() =>
                add({
                  type: 'nodes_paired',
                  node1: selectedLeafNodeIds[0],
                  node2: selectedLeafNodeIds[1],
                })
              }
            />
            <ConditionAction
              icon={<Link2 size={14} />}
              label="Collinear nodes"
              disabled={selectedLeafNodeIds.length !== 3}
              onClick={() =>
                add({
                  type: 'nodes_collinear',
                  node1: selectedLeafNodeIds[0],
                  node2: selectedLeafNodeIds[1],
                  node3: selectedLeafNodeIds[2],
                })
              }
            />
            <ConditionAction
              icon={<LockKeyhole size={14} />}
              label="Fix edge length"
              disabled={!selectedEdge}
              onClick={() => selectedEdge && add({ type: 'edge_length_fixed', edge: selectedEdge.id })}
            />
            <ConditionAction
              icon={<Link2 size={14} />}
              label="Same strain"
              disabled={edgeIds.length !== 2}
              onClick={() => add({ type: 'edges_same_strain', edge1: edgeIds[0], edge2: edgeIds[1] })}
            />
            <ConditionAction
              icon={<Plus size={14} />}
              label="Active path"
              disabled={!selectedPath}
              onClick={() =>
                selectedPath &&
                add({ type: 'path_active', node1: selectedPath.nodes[0], node2: selectedPath.nodes[1] })
              }
            />
          </div>
          {selectedPath && (
            <div className="condition-path-controls">
              <NumberControl label="Angle" value={angle} step={1} onCommit={setAngle} />
              <ConditionAction
                icon={<Plus size={14} />}
                label="Fix angle"
                onClick={() =>
                  add({
                    type: 'path_angle_fixed',
                    node1: selectedPath.nodes[0],
                    node2: selectedPath.nodes[1],
                    angle,
                  })
                }
              />
              <NumberControl label="Quant" value={quant} min={1} step={1} onCommit={setQuant} />
              <NumberControl label="Offset" value={quantOffset} step={1} onCommit={setQuantOffset} />
              <ConditionAction
                icon={<Plus size={14} />}
                label="Quantize"
                onClick={() =>
                  add({
                    type: 'path_angle_quant',
                    node1: selectedPath.nodes[0],
                    node2: selectedPath.nodes[1],
                    quant,
                    quant_offset: quantOffset,
                  })
                }
              />
            </div>
          )}
        </section>

        <section className="condition-section">
          <div className="condition-section__title">List</div>
          {project.conditions.length === 0 ? (
            <div className="empty-note">No conditions</div>
          ) : (
            <div className="condition-list">
              {project.conditions.map((condition) => (
                <button
                  className="condition-item"
                  data-active={isConditionSelected(selection, condition.id) || undefined}
                  data-feasible={condition.isFeasible || undefined}
                  key={condition.id}
                  type="button"
                  onClick={() => select({ kind: 'condition', id: condition.id })}
                >
                  <span>
                    <strong>{conditionTitle(condition.kind)}</strong>
                    <small>{conditionDetail(condition.kind)}</small>
                  </span>
                  <Trash2
                    size={14}
                    role="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      void deleteCondition(condition.id);
                    }}
                  />
                </button>
              ))}
            </div>
          )}
        </section>
      </div>
    </section>
  );
}

function ConditionAction({
  icon,
  label,
  disabled,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button className="condition-action" type="button" disabled={disabled} onClick={onClick}>
      {icon}
      <span>{label}</span>
    </button>
  );
}

function NumberControl({
  label,
  value,
  min,
  max,
  step,
  onCommit,
}: {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step: number;
  onCommit: (value: number) => void;
}) {
  const [draft, setDraft] = useState(String(value));

  useEffect(() => {
    setDraft(String(value));
  }, [value]);

  const commit = () => {
    const parsed = Number.parseFloat(draft);
    if (!Number.isFinite(parsed)) {
      setDraft(String(value));
      return;
    }
    const lowerBounded = min === undefined ? parsed : Math.max(min, parsed);
    const next = max === undefined ? lowerBounded : Math.min(max, lowerBounded);
    if (Math.abs(next - value) > 0.000_001) onCommit(next);
    setDraft(String(next));
  };

  return (
    <label className="condition-number">
      <span>{label}</span>
      <input
        type="number"
        min={min}
        max={max}
        step={step}
        value={draft}
        onChange={(event) => setDraft(event.currentTarget.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === 'Enter') event.currentTarget.blur();
          if (event.key === 'Escape') {
            setDraft(String(value));
            event.currentTarget.blur();
          }
        }}
      />
    </label>
  );
}
