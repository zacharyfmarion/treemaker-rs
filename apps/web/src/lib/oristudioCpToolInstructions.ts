import type {
  OristudioCpActionDefinition,
  OristudioCpCommandActionDefinition,
} from './oristudioCpActions';
import type { OristudioCpCommandDefinition } from './oristudioCpCommands';

export interface OristudioCpToolInstructions {
  intro?: readonly string[];
  steps?: readonly string[];
  notes?: readonly string[];
}

const ORIEDITA_CP_TOOL_INSTRUCTIONS: Record<string, OristudioCpToolInstructions> = {
  angleBisectorAction: {
    intro: ['Draw angle bisector between lines or points.'],
    steps: [
      'Select 2 lines or 3 points forming angle ABC.',
      'Select a target line to extend to, or select the indicator to extend.',
    ],
  },
  axiom5Action: {
    intro: ['Draw a line that places point A to a line while passing through point B.'],
    steps: [
      'Select point A.',
      'Select a line.',
      'Select point B.',
      'Select a target line to extend to, or select the indicator to extend.',
    ],
    notes: [
      'Distance between point A and point B must be equal or greater than projected distance between point B and the line.',
    ],
  },
  axiom7Action: {
    intro: ['Draw a line that places a point to line A, while perpendicular to line B.'],
    steps: [
      'Select a point.',
      'Select lines A and B.',
      'Select a target line to extend to, or select the indicator to extend.',
    ],
    notes: ["The point must not be on line A's span and line B must not be parallel with line A."],
  },
  continuousSymmetricDrawAction: {
    intro: ['Draw a line reflecting through crease lines.'],
    steps: ['Select 2 points to draw a line reflecting and alternating through crease lines.'],
    notes: ['Reflection continues until it hits an edge or a vertex with 3 or more lines.'],
  },
  deg1Action: {
    intro: ['Draw converging lines with angle offset.'],
    steps: [
      'Set offset by selecting a line or selecting 2 points.',
      'Select intersection of converging lines.',
    ],
  },
  deg2Action: {
    intro: ['Draw angle restricted line with selected angle restriction.'],
  },
  deg3Action: {
    intro: ['Draw angle restricted line with offset.'],
    steps: ['Select 2 points to set angle offset.', 'Select a target line to extend to.'],
  },
  del_lAction: {
    intro: ['Delete all coinciding lines by drawing a line.'],
  },
  del_l_XAction: {
    intro: ['Delete all intersecting lines by drawing a line.'],
  },
  doubleSymmetricDrawAction: {
    intro: ['Reflect contacting lines over the drawn line.'],
    steps: ['Draw a line over the end points of lines to reflect them.'],
  },
  drawCreaseFreeAction: {
    intro: ['Free draw straight lines.'],
  },
  drawCreaseRestrictedAction: {
    intro: ['Draw lines snapping to grid and vertices.'],
  },
  drawLineSegmentInternalDivisionRatioAction: {
    intro: ['Draw a line divided by the set ratio.'],
  },
  fishBoneDrawAction: {
    intro: ['Draw perpendicular parallel lines alternating in mountain and valley by drawing a line.'],
    notes: ['The interval of the parallel lines will be equal to the width of grid.'],
  },
  foldableLineDrawAction: {
    intro: ['Draw flat foldable line.'],
    steps: [
      'Select vertex with odd number of connecting lines.',
      'Select flat foldable line.',
      'Select a target line to extend to.',
    ],
    notes: ['Draw anywhere to free draw.'],
  },
  foldableLinePlusGridInputAction: {
    intro: ['Draw flat foldable line.'],
    steps: [
      'Select vertex with odd number of connecting lines.',
      'Select flat foldable line.',
      'Select a target line to extend to.',
    ],
    notes: ['Select a line to extend it.'],
  },
  makeFlatFoldableAction: {
    intro: ['Draw flat foldable line.'],
    steps: [
      'Select vertex with odd number of connecting lines.',
      'Select flat foldable line.',
      'Select a target line to extend to.',
    ],
  },
  lengthenCreaseAction: {
    intro: ['Extend a line with selected color.'],
    steps: [
      'Select or draw a line to select lines.',
      'Select a target line to extend to, or select previous line to extend.',
    ],
  },
  lengthenCrease2Action: {
    intro: ['Extend a line with same color.'],
    steps: [
      'Select or draw a line to select lines.',
      'Select a target line to extend to, or select previous line to extend.',
    ],
  },
  parallelDrawAction: {
    intro: ['Draw parallel line from a point.'],
    steps: [
      'Select a point.',
      'Select a line to make parallel to.',
      'Select a target line to extend to.',
    ],
  },
  perpendicularDrawAction: {
    intro: ['Draw perpendicular line.'],
    steps: ['Select starting point A.', 'Select a target line B.'],
    notes: [
      'A line is drawn from point A to line B perpendicular to line B.',
      'If point A is on line B, select a target line to extend to, or select the indicator to extend.',
    ],
  },
  rabbitEarAction: {
    intro: ['Draw lines toward inner center.'],
    steps: [
      'Select point A.',
      'Select point B.',
      'Select point C.',
      'Three lines toward inner center are drawn.',
    ],
  },
  regularPolygonAction: {
    intro: ['Draw regular polygon by selecting points AB.'],
    notes: ['Points AB will form one side of polygon, which will be drawn clockwise.'],
  },
  selectAction: {
    intro: ['Select lines by drawing a rectangle.'],
  },
  select_lXAction: {
    intro: ['Select overlapping lines by drawing a line.'],
  },
  select_polygonAction: {
    intro: ['Select lines by drawing polygon.'],
    notes: ['Lines completely inside polygon are selected.'],
  },
  selectLassoAction: {
    intro: ['Select lines by drawing freehand.'],
  },
  senbun_b_nyuryokuAction: {
    intro: ['Draw segmented line with selected division.'],
  },
  symmetricDrawAction: {
    intro: ['Draw mirror line by selecting points or lines.'],
    steps: [
      'Select points ABC to mirror line AB over line BC.',
      'Or select lines AB to mirror line A over line B.',
    ],
  },
  unselectAction: {
    intro: ['Deselect lines by drawing a rectangle.'],
  },
  unselect_lXAction: {
    intro: ['Deselect overlapping lines by drawing a line.'],
  },
  unselect_polygonAction: {
    intro: ['Deselect lines by drawing polygon.'],
    notes: ['Lines completely inside polygon are deselected.'],
  },
  unselectLassoAction: {
    intro: ['Deselect lines by drawing freehand.'],
    notes: ['Lines completely inside drawn area are deselected.'],
  },
  v_del_ccAction: {
    intro: ['Delete a vertex on a straight line.'],
  },
  vertexDeleteAction: {
    intro: ['Delete a vertex on a straight line of uniform color.'],
  },
  voronoiAction: {
    intro: ['Draw voronoi pattern by selecting points.'],
    notes: [
      'Selecting points again will cancel them.',
      'Select apply temporary lines to change them to crease lines.',
    ],
  },
};

export function instructionsForCpAction(
  action: OristudioCpActionDefinition | null | undefined
): OristudioCpToolInstructions | null {
  if (!action) return null;
  const fallbackUpstream =
    isCommandAction(action) ? action.command.upstream : undefined;
  return instructionsForOrieditaAction(action.upstreamAction) ?? instructionsForOrieditaAction(fallbackUpstream);
}

export function instructionsForCpTool(
  action: OristudioCpActionDefinition | null | undefined,
  command: OristudioCpCommandDefinition | null | undefined
): OristudioCpToolInstructions | null {
  return instructionsForCpAction(action) ?? instructionsForOrieditaAction(command?.upstream);
}

export function instructionsForOrieditaAction(
  upstreamAction: string | null | undefined
): OristudioCpToolInstructions | null {
  if (!upstreamAction) return null;
  return ORIEDITA_CP_TOOL_INSTRUCTIONS[upstreamAction] ?? null;
}

function isCommandAction(
  action: OristudioCpActionDefinition
): action is OristudioCpCommandActionDefinition {
  return action.kind === 'command';
}
