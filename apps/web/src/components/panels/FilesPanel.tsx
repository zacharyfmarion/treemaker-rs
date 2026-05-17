import { FileText, FolderOpen, Save } from 'lucide-react';
import { Button } from '../ui/Button';
import { useWorkspaceStore } from '../../store/workspaceStore';

export function FilesPanel() {
  const createNewProject = useWorkspaceStore((state) => state.createNewProject);

  return (
    <section className="panel-shell files-panel">
      <div className="panel-toolbar">
        <span className="panel-title">Files</span>
      </div>
      <div className="panel-body files-panel__body">
        <div className="file-actions">
          <Button size="sm" variant="secondary" onClick={() => void createNewProject()}>
            <FileText size={14} />
            New
          </Button>
          <Button size="sm" variant="secondary" disabled>
            <FolderOpen size={14} />
            Import
          </Button>
          <Button size="sm" variant="primary" disabled>
            <Save size={14} />
            Save
          </Button>
        </div>
        <div className="example-list">
          <button
            type="button"
            className="example-item"
            onClick={() => void createNewProject()}
          >
            <span className="example-item__title">Three terminal flaps</span>
            <span className="example-item__meta">Scale 0.100 | Nodes 4</span>
          </button>
          <button type="button" className="example-item" disabled>
            <span className="example-item__title">Mirrored fork</span>
            <span className="example-item__meta">Symmetry | Nodes 5</span>
          </button>
          <button type="button" className="example-item" disabled>
            <span className="example-item__title">Asymmetric antler</span>
            <span className="example-item__meta">Branching | Nodes 10</span>
          </button>
        </div>
      </div>
    </section>
  );
}
