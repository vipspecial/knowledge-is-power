import DOMPurify from "dompurify";

let mermaidLoader: Promise<typeof import("mermaid")["default"]> | null = null;
let diagramCounter = 0;

function loadMermaid(): Promise<typeof import("mermaid")["default"]> {
  if (!mermaidLoader) {
    mermaidLoader = import("mermaid").then((module) => {
      module.default.initialize({
        startOnLoad: false,
        securityLevel: "strict",
      });
      return module.default;
    });
  }
  return mermaidLoader;
}

/** 渲染 Mermaid 源码为净化后的 SVG；语法错误时抛出异常由调用方保留源码展示。 */
export async function renderMermaidSvg(source: string): Promise<string> {
  const mermaid = await loadMermaid();
  const { svg } = await mermaid.render(`mermaid-diagram-${++diagramCounter}`, source);
  return DOMPurify.sanitize(svg);
}
