import { useEffect, useMemo, useState, type ReactNode } from 'react';
import {
  ChevronRight,
  FileQuestionMark,
  FileText,
  FlipHorizontal2,
  FolderOpen,
  Link2,
  LockKeyhole,
  Plus,
  Ruler,
  ScanLine,
  Trash2,
} from 'lucide-react';
import { handleMenuAction } from '../../commands/menuActions';
import type { ConditionKind } from '../../engine/types';
import type { TreeCondition } from '../../lib/sampleProject';
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
  selectedConditionIds,
  selectedEdgeIds,
  selectedNodeIds,
  selectedPathIds,
  toggleConditionSelection,
} from '../../lib/selection';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { Button } from '../ui/Button';
import { IconButton } from '../ui/IconButton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/Select';

export function ConditionsPanel() {
  const project = useWorkspaceStore((state) => state.project);
  const documentMode = useWorkspaceStore((state) => state.documentMode);
  const selection = useWorkspaceStore((state) => state.selection);
  const select = useWorkspaceStore((state) => state.select);
  const updatePaper = useWorkspaceStore((state) => state.updatePaper);
  const setSymmetry = useWorkspaceStore((state) => state.setSymmetry);
  const addCondition = useWorkspaceStore((state) => state.addCondition);
  const updateCondition = useWorkspaceStore((state) => state.updateCondition);
  const deleteCondition = useWorkspaceStore((state) => state.deleteCondition);
  const deleteConditionsForSelectedNodes = useWorkspaceStore((state) => state.deleteConditionsForSelectedNodes);
  const deleteConditionsForSelectedEdges = useWorkspaceStore((state) => state.deleteConditionsForSelectedEdges);
  const deleteConditionsForSelectedPaths = useWorkspaceStore((state) => state.deleteConditionsForSelectedPaths);
  const clearConditions = useWorkspaceStore((state) => state.clearConditions);
  const nodeIds = selectedNodeIds(selection);
  const edgeIds = selectedEdgeIds(selection);
  const conditionIds = selectedConditionIds(selection);
  const pathIds = selectedPathIds(selection);
  const selectedNode = nodeIds.length === 1 ? project.nodes.find((node) => node.id === nodeIds[0]) : null;
  const selectedEdge = edgeIds.length === 1 ? project.edges.find((edge) => edge.id === edgeIds[0]) : null;
  const selectedPath =
    selection.kind === 'path' ? project.paths.find((path) => path.id === selection.id) : null;
  const editedCondition =
    conditionIds.length === 1
      ? project.conditions.find((condition) => condition.id === conditionIds[0])
      : null;
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

  if (documentMode === 'crease-pattern') {
    return (
      <section className="panel-shell conditions-panel">
        <div className="panel-toolbar">
          <span className="panel-title">Conditions</span>
        </div>
        <div className="panel-body document-mode-empty">
          <div className="document-mode-empty__icon" aria-hidden="true">
            <FileQuestionMark size={30} />
          </div>
          <span className="document-mode-empty__message">
            Imported crease patterns do not have editable tree conditions.
          </span>
          <div className="document-mode-empty__actions">
            <Button size="sm" variant="primary" onClick={() => void handleMenuAction('view.creasePattern')}>
              <ScanLine size={14} />
              View CP
            </Button>
            <Button size="sm" variant="secondary" onClick={() => void handleMenuAction('file.new')}>
              <FileText size={14} />
              New Tree
            </Button>
            <Button size="sm" variant="secondary" onClick={() => void handleMenuAction('file.open')}>
              <FolderOpen size={14} />
              Open
            </Button>
          </div>
        </div>
      </section>
    );
  }

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
          <div className="condition-actions">
            <ConditionAction
              icon={<Trash2 size={14} />}
              label="Remove node conditions"
              disabled={nodeIds.length === 0}
              onClick={() => void deleteConditionsForSelectedNodes()}
            />
            <ConditionAction
              icon={<Trash2 size={14} />}
              label="Remove edge conditions"
              disabled={edgeIds.length === 0}
              onClick={() => void deleteConditionsForSelectedEdges()}
            />
            <ConditionAction
              icon={<Trash2 size={14} />}
              label="Remove path conditions"
              disabled={pathIds.length === 0}
              onClick={() => void deleteConditionsForSelectedPaths()}
            />
          </div>
        </section>

        {editedCondition && (
          <section className="condition-section">
            <div className="condition-section__title">Editor</div>
            <ConditionEditor
              condition={editedCondition}
              onUpdate={(kind) => void updateCondition(editedCondition.id, kind)}
            />
          </section>
        )}

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
                  onClick={(event) => {
                    if (event.metaKey || event.ctrlKey || event.shiftKey) {
                      select(toggleConditionSelection(selection, condition.id));
                      return;
                    }
                    select({ kind: 'condition', id: condition.id });
                  }}
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

function ConditionEditor({
  condition,
  onUpdate,
}: {
  condition: TreeCondition;
  onUpdate: (kind: ConditionKind) => void;
}) {
  const kind = condition.kind;
  const update = (patch: Partial<ConditionKind>) => onUpdate({ ...kind, ...patch } as ConditionKind);
  const updateInt = (key: string, value: number) => update({ [key]: Math.max(1, Math.round(value)) } as Partial<ConditionKind>);
  const updateNumber = (key: string, value: number) => update({ [key]: value } as Partial<ConditionKind>);
  const updateBool = (key: string, value: boolean) => update({ [key]: value } as Partial<ConditionKind>);

  switch (kind.type) {
    case 'node_combo':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node" value={kind.node} min={1} step={1} onCommit={(value) => updateInt('node', value)} />
          <ToggleControl label="Sym line" checked={kind.to_symmetry_line} onChange={(value) => updateBool('to_symmetry_line', value)} />
          <ToggleControl label="Paper edge" checked={kind.to_paper_edge} onChange={(value) => updateBool('to_paper_edge', value)} />
          <ToggleControl label="Corner" checked={kind.to_paper_corner} onChange={(value) => updateBool('to_paper_corner', value)} />
          <ToggleControl label="Fix X" checked={kind.x_fixed} onChange={(value) => updateBool('x_fixed', value)} />
          <NumberControl label="X" value={kind.x_fix_value} step={0.01} onCommit={(value) => updateNumber('x_fix_value', value)} />
          <ToggleControl label="Fix Y" checked={kind.y_fixed} onChange={(value) => updateBool('y_fixed', value)} />
          <NumberControl label="Y" value={kind.y_fix_value} step={0.01} onCommit={(value) => updateNumber('y_fix_value', value)} />
        </div>
      );
    case 'node_fixed':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node" value={kind.node} min={1} step={1} onCommit={(value) => updateInt('node', value)} />
          <ToggleControl label="Fix X" checked={kind.x_fixed} onChange={(value) => updateBool('x_fixed', value)} />
          <NumberControl label="X" value={kind.x_fix_value} step={0.01} onCommit={(value) => updateNumber('x_fix_value', value)} />
          <ToggleControl label="Fix Y" checked={kind.y_fixed} onChange={(value) => updateBool('y_fixed', value)} />
          <NumberControl label="Y" value={kind.y_fix_value} step={0.01} onCommit={(value) => updateNumber('y_fix_value', value)} />
        </div>
      );
    case 'node_on_corner':
    case 'node_on_edge':
    case 'node_symmetric':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node" value={kind.node} min={1} step={1} onCommit={(value) => updateInt('node', value)} />
        </div>
      );
    case 'nodes_paired':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
        </div>
      );
    case 'nodes_collinear':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
          <NumberControl label="Node 3" value={kind.node3} min={1} step={1} onCommit={(value) => updateInt('node3', value)} />
        </div>
      );
    case 'edge_length_fixed':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Edge" value={kind.edge} min={1} step={1} onCommit={(value) => updateInt('edge', value)} />
        </div>
      );
    case 'edges_same_strain':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Edge 1" value={kind.edge1} min={1} step={1} onCommit={(value) => updateInt('edge1', value)} />
          <NumberControl label="Edge 2" value={kind.edge2} min={1} step={1} onCommit={(value) => updateInt('edge2', value)} />
        </div>
      );
    case 'path_combo':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
          <ToggleControl label="Fix angle" checked={kind.is_angle_fixed} onChange={(value) => updateBool('is_angle_fixed', value)} />
          <NumberControl label="Angle" value={kind.angle} step={1} onCommit={(value) => updateNumber('angle', value)} />
          <ToggleControl label="Quantize" checked={kind.is_angle_quant} onChange={(value) => updateBool('is_angle_quant', value)} />
          <NumberControl label="Quant" value={kind.quant} min={1} step={1} onCommit={(value) => updateInt('quant', value)} />
          <NumberControl label="Offset" value={kind.quant_offset} step={1} onCommit={(value) => updateNumber('quant_offset', value)} />
        </div>
      );
    case 'path_active':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
        </div>
      );
    case 'path_angle_fixed':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
          <NumberControl label="Angle" value={kind.angle} step={1} onCommit={(value) => updateNumber('angle', value)} />
        </div>
      );
    case 'path_angle_quant':
      return (
        <div className="condition-advanced__body">
          <NumberControl label="Node 1" value={kind.node1} min={1} step={1} onCommit={(value) => updateInt('node1', value)} />
          <NumberControl label="Node 2" value={kind.node2} min={1} step={1} onCommit={(value) => updateInt('node2', value)} />
          <NumberControl label="Quant" value={kind.quant} min={1} step={1} onCommit={(value) => updateInt('quant', value)} />
          <NumberControl label="Offset" value={kind.quant_offset} step={1} onCommit={(value) => updateNumber('quant_offset', value)} />
        </div>
      );
  }
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

function ToggleControl({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="condition-toggle">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
    </label>
  );
}
