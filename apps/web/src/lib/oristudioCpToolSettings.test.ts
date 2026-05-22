import { describe, expect, it } from 'vitest';
import {
  DEFAULT_ORISTUDIO_CP_TOOL_OPTIONS,
  cpToolSettingGroupsForOperation,
  evaluateOrieditaRatioExpression,
} from './oristudioCpToolSettings';

describe('oristudioCpToolSettings', () => {
  it('uses Oriedita exact-ratio defaults for line ratio division', () => {
    const ratio = evaluateOrieditaRatioExpression(
      DEFAULT_ORISTUDIO_CP_TOOL_OPTIONS.divisionRatio
    );

    expect(ratio.ratioS).toBe(1);
    expect(ratio.ratioT).toBeCloseTo(Math.sqrt(2));
  });

  it('maps operations to contextual option groups', () => {
    expect(cpToolSettingGroupsForOperation('LineSegmentDivision')).toEqual([
      'division-count',
    ]);
    expect(cpToolSettingGroupsForOperation('LineSegmentRatioSet')).toEqual([
      'division-ratio',
    ]);
    expect(cpToolSettingGroupsForOperation('ReplaceLineTypeSelect')).toEqual([
      'replace-line-type',
    ]);
    expect(cpToolSettingGroupsForOperation('FixInaccurate')).toEqual(['fix-precision']);
    expect(cpToolSettingGroupsForOperation('VoronoiCreate')).toEqual(['apply-lines']);
    expect(cpToolSettingGroupsForOperation('CircleChangeColor')).toEqual([
      'custom-circle-color',
    ]);
    expect(cpToolSettingGroupsForOperation('DisplayLengthBetweenPoints1')).toEqual([
      'measurement-readout',
    ]);
  });

  it('includes line color and angle system settings for angle-restricted drawing', () => {
    expect(cpToolSettingGroupsForOperation('DrawCreaseAngleRestricted')).toEqual([
      'line-color',
      'angle-system',
      'candidate-choice',
    ]);
  });
});
