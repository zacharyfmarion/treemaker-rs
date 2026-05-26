type StartScreenRequestHandler = () => boolean | Promise<boolean>;

let startScreenRequestHandler: StartScreenRequestHandler | null = null;

export function registerStartScreenRequestHandler(handler: StartScreenRequestHandler): () => void {
  startScreenRequestHandler = handler;
  return () => {
    if (startScreenRequestHandler === handler) {
      startScreenRequestHandler = null;
    }
  };
}

export async function requestStartScreen(): Promise<boolean> {
  return (await startScreenRequestHandler?.()) ?? false;
}
