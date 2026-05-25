# Oriedita CP Tool Rail Alignment

## Goal

Align the editable crease-pattern tool rail with Oriedita's Drawing tab order
while keeping the M / V / E / A line-type controls in the bottom toolbar.

## Approach

- Add explicit rail ordering and labels to the CP action registry.
- Expose Oriedita dropdown entries as individual rail buttons so migrated tools
  are visible without extra discovery.
- Keep line-type actions bottom-toolbar only.
- Use Oriedita's `Icons2.ttf` glyphs for rail icons and update the rail grid
  and icon sizing to match Oriedita's four-across tool density.
- Update frontend tests for registry order, bottom-toolbar line types, and
  rendered rail controls.

## Affected Areas

- `apps/web/src/lib/oristudioCpActions.ts`
- `apps/web/src/lib/oristudioCpActions.test.ts`
- `apps/web/src/components/panels/CpToolRail.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/styles/theme.css`

## Checklist

- [x] Add explicit Oriedita rail action ordering.
- [x] Keep M / V / E / A in the bottom toolbar only.
- [x] Add or relabel visible rail actions to match Oriedita Drawing tab names.
- [x] Update rail styling to four columns with larger icons.
- [x] Update frontend tests.
- [x] Run web validation.
