import { describe, expect, it } from 'vitest';
import type { OristudioCpDocumentSnapshot } from '../engine/oristudioCpTypes';
import { ORISTUDIO_CP_COMMANDS } from './oristudioCpCommands';
import {
  ORISTUDIO_CP_SYMMETRY_POLICIES,
  reflectedCpCommandPayloads,
  type OristudioCpSymmetryState,
} from './oristudioCpSymmetry';

const verticalSymmetry: OristudioCpSymmetryState = {
  enabled: true,
  showAxis: true,
  preset: 'book',
  axis: {
    loc: { x: 0, y: 0 },
    angle: 90,
  },
};

function cpDocument(): OristudioCpDocumentSnapshot {
  return {
    title: 'Symmetric CP',
    metadata: {},
    crease_pattern: {
      line_segments: [
        {
          a: { x: 1, y: 0 },
          b: { x: 1, y: 2 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          a: { x: -1, y: 0 },
          b: { x: -1, y: 2 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          a: { x: 1, y: 0 },
          b: { x: 2, y: 1 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          a: { x: -1, y: 0 },
          b: { x: -2, y: 1 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          a: { x: 1, y: 3 },
          b: { x: 2, y: 3 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          a: { x: -1, y: 3 },
          b: { x: -2, y: 3 },
          color: 'Red1',
          active: 'Inactive0',
          selected: 0,
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
      ],
      circles: [
        {
          x: 2,
          y: 0,
          r: 1,
          color: 'Cyan3',
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
        {
          x: -2,
          y: 0,
          r: 1,
          color: 'Cyan3',
          customized: 0,
          customized_color: { red: 0, green: 0, blue: 0 },
        },
      ],
      points: [],
      aux_line_segments: [],
      texts: [
        { x: 3, y: 0, text: 'note' },
        { x: -3, y: 0, text: 'note' },
      ],
      grid: {
        interval_grid_size: 4,
        grid_size: 8,
        grid_xa: 1,
        grid_xb: 0,
        grid_xc: 1,
        grid_ya: 1,
        grid_yb: 0,
        grid_yc: 1,
        grid_angle: 90,
        base_state: 'WithinPaper',
        vertical_scale_position: 0,
        horizontal_scale_position: 0,
        draw_diagonal_gridlines: false,
      },
    },
  };
}

describe('oristudio CP symmetry', () => {
  it('documents symmetry behavior for every ready CP command', () => {
    const missingPolicies = ORISTUDIO_CP_COMMANDS.filter(
      (command) => command.uiStatus === 'ready'
    )
      .map((command) => command.operationId)
      .filter((operationId) => !(operationId in ORISTUDIO_CP_SYMMETRY_POLICIES));

    expect(missingPolicies).toEqual([]);
  });

  it('adds reflected point payloads for geometric commands', () => {
    const payloads = reflectedCpCommandPayloads(
      'DrawCreaseFree',
      cpDocument(),
      {
        points: [
          { x: 10, y: 1 },
          { x: 20, y: 1 },
        ],
        line_color: 'Red1',
      },
      verticalSymmetry
    );

    expect(payloads).toHaveLength(2);
    expect(payloads[1]?.points?.[0]?.x).toBeCloseTo(-10);
    expect(payloads[1]?.points?.[0]?.y).toBeCloseTo(1);
    expect(payloads[1]?.points?.[1]?.x).toBeCloseTo(-20);
    expect(payloads[1]?.points?.[1]?.y).toBeCloseTo(1);
  });

  it('expands selected line ids to their reflected partners', () => {
    const payloads = reflectedCpCommandPayloads(
      'CreaseMakeMountain',
      cpDocument(),
      {
        line_ids: [1],
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([{ line_ids: [1, 2] }]);
  });

  it('expands selected circle ids to their reflected partners', () => {
    const payloads = reflectedCpCommandPayloads(
      'CircleChangeColor',
      cpDocument(),
      {
        circle_ids: [1],
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([{ circle_ids: [1, 2] }]);
  });

  it('keeps mirrored selection-box commands additive after the primary command', () => {
    const payloads = reflectedCpCommandPayloads(
      'CreaseSelect',
      cpDocument(),
      {
        points: [
          { x: 3, y: 0 },
          { x: 5, y: 2 },
        ],
        replace_selection: true,
      },
      verticalSymmetry
    );

    expect(payloads).toHaveLength(2);
    expect(payloads[0]?.replace_selection).toBe(true);
    expect(payloads[1]?.replace_selection).toBe(false);
  });

  it('mirrors ordered line operands without changing command arity', () => {
    const payloads = reflectedCpCommandPayloads(
      'SquareBisector',
      cpDocument(),
      {
        line_ids: [1, 3, 5],
        line_color: 'Red1',
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([
      { line_ids: [1, 3, 5], line_color: 'Red1' },
      { line_ids: [2, 4, 6], line_color: 'Red1' },
    ]);
  });

  it('does not mirror ordered line operands when the mirrored set is incomplete', () => {
    const document = cpDocument();
    document.crease_pattern.line_segments = document.crease_pattern.line_segments.slice(0, 5);

    const payloads = reflectedCpCommandPayloads(
      'SquareBisector',
      document,
      {
        line_ids: [1, 3, 5],
        line_color: 'Red1',
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([{ line_ids: [1, 3, 5], line_color: 'Red1' }]);
  });

  it('mirrors ordered circle operands without expanding them into one payload', () => {
    const payloads = reflectedCpCommandPayloads(
      'CircleDrawConcentricTwoCircleSelect',
      cpDocument(),
      {
        circle_ids: [1, 2],
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([{ circle_ids: [1, 2] }, { circle_ids: [2, 1] }]);
  });

  it('expands selected text ids to their reflected partners for text edits', () => {
    const payloads = reflectedCpCommandPayloads(
      'Text',
      cpDocument(),
      {
        text_action: 'SetContent',
        text_ids: [1],
        text_content: 'updated',
      },
      verticalSymmetry
    );

    expect(payloads).toEqual([
      { text_action: 'SetContent', text_ids: [1, 2], text_content: 'updated' },
    ]);
  });
});
