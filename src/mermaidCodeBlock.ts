import CodeBlock from "@tiptap/extension-code-block";
import type { NodeViewRendererProps } from "@tiptap/core";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import type { ViewMutationRecord } from "@tiptap/pm/view";
import { renderMermaidSvg } from "./mermaid";

const RENDER_DEBOUNCE_MS = 600;

/**
 * 代码块扩展：`mermaid` 语言时在源码上方渲染图表预览，源码始终保留可编辑；
 * 其他语言保持原生代码块表现。
 */
export const MermaidCodeBlock = CodeBlock.extend({
  addNodeView() {
    return (props: NodeViewRendererProps) => new MermaidBlockView(props);
  },
});

class MermaidBlockView {
  readonly dom: HTMLElement;
  readonly contentDOM: HTMLElement;
  private readonly diagram: HTMLElement;
  private readonly pre: HTMLElement;
  private node: ProseMirrorNode;
  private lastSource = "";
  private timer: number | undefined;
  private destroyed = false;

  constructor(props: NodeViewRendererProps) {
    this.node = props.node;
    this.dom = document.createElement("div");
    this.dom.className = "mermaid-code-block";
    this.diagram = document.createElement("div");
    this.diagram.className = "mermaid-diagram";
    this.pre = document.createElement("pre");
    const code = document.createElement("code");
    this.pre.appendChild(code);
    this.contentDOM = code;
    this.dom.append(this.diagram, this.pre);
    this.sync();
  }

  update(node: ProseMirrorNode): boolean {
    if (node.type !== this.node.type) return false;
    this.node = node;
    this.sync();
    return true;
  }

  ignoreMutation(mutation: ViewMutationRecord): boolean {
    return !this.contentDOM.contains(mutation.target);
  }

  destroy(): void {
    this.destroyed = true;
    window.clearTimeout(this.timer);
  }

  private sync(): void {
    const isMermaid = this.node.attrs.language === "mermaid";
    this.dom.classList.toggle("is-mermaid", isMermaid);
    if (!isMermaid) {
      this.lastSource = "";
      this.diagram.replaceChildren();
      return;
    }
    const source = this.node.textContent;
    if (source === this.lastSource) return;
    this.lastSource = source;
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => void this.render(source), RENDER_DEBOUNCE_MS);
  }

  private async render(source: string): Promise<void> {
    if (this.destroyed || source !== this.lastSource || !source.trim()) return;
    try {
      const svg = await renderMermaidSvg(source);
      if (this.destroyed || source !== this.lastSource) return;
      this.diagram.innerHTML = svg;
      this.diagram.classList.remove("has-error");
    } catch {
      if (this.destroyed || source !== this.lastSource) return;
      this.diagram.replaceChildren();
      this.diagram.classList.add("has-error");
      const hint = document.createElement("span");
      hint.textContent = "图表语法有误，已保留源码";
      this.diagram.appendChild(hint);
    }
  }
}
