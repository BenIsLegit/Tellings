declare global {
  namespace App {}

  /** The frame renderer, loaded from renderer/frame.js. */
  // eslint-disable-next-line no-var
  var PVC: {
    create(
      host: HTMLElement,
      plan: unknown,
      opts?: { css?: string; shadow?: boolean; picture?: (file: string) => string | Promise<string> },
    ): {
      el: HTMLElement;
      ready: Promise<unknown>;
      show(index: number, o?: { line?: number | null }): Promise<{ font: number; overflow: boolean }>;
      preload(): Promise<unknown>;
      fitTo(width: number, height: number): number;
    };
    fontCss(urls: { cardo400: string; cardo700: string; fira400: string; fira500: string }): string;
    installFonts(css: string): void;
  };
}

export {};
