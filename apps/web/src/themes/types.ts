export interface ThemeTokens {
  'bg.primary': string;
  'bg.secondary': string;
  'bg.tertiary': string;
  'bg.surface': string;
  'bg.canvas': string;
  'bg.canvasGrid': string;
  'text.primary': string;
  'text.secondary': string;
  'text.muted': string;
  'text.inverse': string;
  'accent.primary': string;
  'accent.hover': string;
  'border.default': string;
  'border.active': string;
  'status.danger': string;
  'status.success': string;
  'status.errorBg': string;
  'port.image': string;
  'port.float': string;
  'port.int': string;
  'port.bool': string;
  'port.color': string;
  'port.mask': string;
  'port.field': string;
  'frame.default': string;
  'node.bg': string;
  'node.selected': string;
  'node.shadow': string;
  'node.shadowSelected': string;
  'node.header.input': string;
  'node.header.output': string;
  'node.header.color': string;
  'node.header.filter': string;
  'node.header.composite': string;
  'node.header.transform': string;
  'node.header.generator': string;
  'node.header.matte': string;
  'node.header.group': string;
  'node.header.groupInput': string;
  'node.header.groupOutput': string;
  'node.header.text': string;
  'slider.fill': string;
  'slider.fillHover': string;
  'slider.bg': string;
  'shadow.overlay': string;
  'shadow.contextMenu': string;
  'minimap.mask': string;
}

export interface SyntaxColors {
  comment: string;
  keyword: string;
  type: string;
  variable: string;
  parameter: string;
  port: string;
  function: string;
  number: string;
  string: string;
  operator: string;
  stringEscape: string;
  foreground: string;
}

export interface TreeMakerTheme {
  name: string;
  type: 'dark' | 'light';
  colors: ThemeTokens;
  syntaxColors: SyntaxColors;
}

export function tokenToCssVar(token: string): string {
  return `--${token.replace(/\./g, '-')}`;
}
