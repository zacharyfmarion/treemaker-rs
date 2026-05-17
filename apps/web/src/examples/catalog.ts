import asymmetricAntler from '../../../../tests/fixtures/generated/asymmetric-antler-optimized.tmd5?raw';
import mirroredFork from '../../../../tests/fixtures/generated/mirrored-fork-optimized.tmd5?raw';
import triad from '../../../../tests/fixtures/generated/triad-optimized.tmd5?raw';

export interface ExampleProject {
  id: string;
  title: string;
  meta: string;
  filename: string;
  text: string;
}

export const EXAMPLE_PROJECTS: ExampleProject[] = [
  {
    id: 'triad',
    title: 'Three terminal flaps',
    meta: 'Optimized triad | Nodes 4',
    filename: 'triad-optimized.tmd5',
    text: triad,
  },
  {
    id: 'mirrored-fork',
    title: 'Mirrored fork',
    meta: 'Symmetry | Nodes 5',
    filename: 'mirrored-fork-optimized.tmd5',
    text: mirroredFork,
  },
  {
    id: 'asymmetric-antler',
    title: 'Asymmetric antler',
    meta: 'Branching | Nodes 10',
    filename: 'asymmetric-antler-optimized.tmd5',
    text: asymmetricAntler,
  },
];

export function getExampleProject(id: string): ExampleProject | undefined {
  return EXAMPLE_PROJECTS.find((example) => example.id === id);
}
