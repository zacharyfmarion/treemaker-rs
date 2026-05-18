export class GpuMath {
  readonly gl: WebGL2RenderingContext | WebGLRenderingContext;
  private programs = new Map<string, WebGLProgram>();

  constructor(gl: WebGL2RenderingContext | WebGLRenderingContext) {
    this.gl = gl;
  }

  static fromCanvas(canvas: HTMLCanvasElement | OffscreenCanvas): GpuMath | null {
    const gl =
      canvas.getContext('webgl2') ??
      canvas.getContext('webgl', { antialias: true, preserveDrawingBuffer: true });
    return gl ? new GpuMath(gl as WebGL2RenderingContext | WebGLRenderingContext) : null;
  }

  compileProgram(name: string, vertexSource: string, fragmentSource: string): WebGLProgram {
    const vertex = this.compileShader(this.gl.VERTEX_SHADER, vertexSource);
    const fragment = this.compileShader(this.gl.FRAGMENT_SHADER, fragmentSource);
    const program = this.gl.createProgram();
    if (!program) throw new Error('Unable to create WebGL program');
    this.gl.attachShader(program, vertex);
    this.gl.attachShader(program, fragment);
    this.gl.linkProgram(program);
    if (!this.gl.getProgramParameter(program, this.gl.LINK_STATUS)) {
      const info = this.gl.getProgramInfoLog(program) ?? 'unknown WebGL link error';
      this.gl.deleteProgram(program);
      throw new Error(info);
    }
    this.programs.set(name, program);
    return program;
  }

  clear(red = 1, green = 1, blue = 1, alpha = 1): void {
    this.gl.clearColor(red, green, blue, alpha);
    this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
  }

  dispose(): void {
    for (const program of this.programs.values()) {
      this.gl.deleteProgram(program);
    }
    this.programs.clear();
  }

  private compileShader(type: number, source: string): WebGLShader {
    const shader = this.gl.createShader(type);
    if (!shader) throw new Error('Unable to create WebGL shader');
    this.gl.shaderSource(shader, source);
    this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      const info = this.gl.getShaderInfoLog(shader) ?? 'unknown WebGL compile error';
      this.gl.deleteShader(shader);
      throw new Error(info);
    }
    return shader;
  }
}

export function detectWebGlSupport(): boolean {
  if (typeof document === 'undefined') return false;
  const canvas = document.createElement('canvas');
  return Boolean(canvas.getContext('webgl2') ?? canvas.getContext('webgl'));
}
