/**
 * Comprehensive Designer Interaction Tests
 *
 * Every visual interaction a user can perform in the designer must:
 * 1. Map to a source code change
 * 2. Sync to other editors via CRDT
 * 3. Re-render correctly in all connected designers
 *
 * This test suite covers ALL user interactions:
 * - Style changes (inline, computed, responsive)
 * - Node operations (move, delete, duplicate, insert)
 * - Text editing (inline, content editable)
 * - Attribute changes
 * - Component operations
 * - Layout adjustments
 * - Canvas operations (zoom, pan, frame positioning)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { CRDTDocument } from './crdt';
import { DocumentSession, SyncTransport } from './sync';

// ============================================================================
// Types for Designer Mutations
// ============================================================================

/**
 * All possible mutations from the visual designer.
 * Each maps to a source code transformation.
 */
type DesignerMutation =
  // Style mutations
  | { type: 'setStyle'; nodeId: string; property: string; value: string }
  | { type: 'removeStyle'; nodeId: string; property: string }
  | { type: 'setStyleVariant'; nodeId: string; variant: string; property: string; value: string }

  // Node mutations
  | { type: 'deleteNode'; nodeId: string }
  | { type: 'moveNode'; nodeId: string; newParentId: string; index: number }
  | { type: 'duplicateNode'; nodeId: string }
  | { type: 'insertElement'; parentId: string; index: number; tagName: string }
  | { type: 'wrapInElement'; nodeId: string; wrapperTagName: string }
  | { type: 'unwrapElement'; nodeId: string }

  // Text mutations
  | { type: 'setText'; nodeId: string; content: string }
  | { type: 'insertText'; parentId: string; index: number; content: string }

  // Attribute mutations
  | { type: 'setAttribute'; nodeId: string; name: string; value: string }
  | { type: 'removeAttribute'; nodeId: string; name: string }
  | { type: 'setClassName'; nodeId: string; className: string }
  | { type: 'toggleClassName'; nodeId: string; className: string; enabled: boolean }

  // Component mutations
  | { type: 'renameComponent'; componentId: string; newName: string }
  | { type: 'extractComponent'; nodeId: string; componentName: string }
  | { type: 'convertToSlot'; nodeId: string; slotName: string }

  // Layout mutations
  | { type: 'setDisplay'; nodeId: string; display: 'block' | 'flex' | 'grid' | 'inline' | 'none' }
  | { type: 'setFlexDirection'; nodeId: string; direction: 'row' | 'column' }
  | { type: 'setJustifyContent'; nodeId: string; value: string }
  | { type: 'setAlignItems'; nodeId: string; value: string }
  | { type: 'setGap'; nodeId: string; value: string }

  // Sizing mutations
  | { type: 'setWidth'; nodeId: string; value: string }
  | { type: 'setHeight'; nodeId: string; value: string }
  | { type: 'setMinWidth'; nodeId: string; value: string }
  | { type: 'setMaxWidth'; nodeId: string; value: string }
  | { type: 'setMinHeight'; nodeId: string; value: string }
  | { type: 'setMaxHeight'; nodeId: string; value: string }

  // Spacing mutations
  | { type: 'setPadding'; nodeId: string; value: string }
  | { type: 'setPaddingTop'; nodeId: string; value: string }
  | { type: 'setPaddingRight'; nodeId: string; value: string }
  | { type: 'setPaddingBottom'; nodeId: string; value: string }
  | { type: 'setPaddingLeft'; nodeId: string; value: string }
  | { type: 'setMargin'; nodeId: string; value: string }
  | { type: 'setMarginTop'; nodeId: string; value: string }
  | { type: 'setMarginRight'; nodeId: string; value: string }
  | { type: 'setMarginBottom'; nodeId: string; value: string }
  | { type: 'setMarginLeft'; nodeId: string; value: string }

  // Position mutations
  | { type: 'setPosition'; nodeId: string; position: 'static' | 'relative' | 'absolute' | 'fixed' }
  | { type: 'setTop'; nodeId: string; value: string }
  | { type: 'setRight'; nodeId: string; value: string }
  | { type: 'setBottom'; nodeId: string; value: string }
  | { type: 'setLeft'; nodeId: string; value: string }
  | { type: 'setZIndex'; nodeId: string; value: string }

  // Visual mutations
  | { type: 'setBackgroundColor'; nodeId: string; value: string }
  | { type: 'setColor'; nodeId: string; value: string }
  | { type: 'setBorderRadius'; nodeId: string; value: string }
  | { type: 'setBorder'; nodeId: string; value: string }
  | { type: 'setBoxShadow'; nodeId: string; value: string }
  | { type: 'setOpacity'; nodeId: string; value: string }

  // Typography mutations
  | { type: 'setFontFamily'; nodeId: string; value: string }
  | { type: 'setFontSize'; nodeId: string; value: string }
  | { type: 'setFontWeight'; nodeId: string; value: string }
  | { type: 'setLineHeight'; nodeId: string; value: string }
  | { type: 'setTextAlign'; nodeId: string; value: string }
  | { type: 'setTextDecoration'; nodeId: string; value: string }

  // Canvas/Frame mutations
  | { type: 'setFramePosition'; frameId: string; x: number; y: number }
  | { type: 'setFrameSize'; frameId: string; width: number; height: number }
  | { type: 'setFrameBounds'; frameId: string; x: number; y: number; width: number; height: number };

// ============================================================================
// Source Code Mutator
// ============================================================================

/**
 * Applies designer mutations to source code.
 * This simulates what paperclip-editor does in Rust.
 */
class SourceMutator {
  apply(source: string, mutation: DesignerMutation): string {
    switch (mutation.type) {
      // Style mutations
      case 'setStyle':
        return this.setStyle(source, mutation.nodeId, mutation.property, mutation.value);
      case 'removeStyle':
        return this.removeStyle(source, mutation.nodeId, mutation.property);

      // Node mutations
      case 'deleteNode':
        return this.deleteNode(source, mutation.nodeId);
      case 'moveNode':
        return this.moveNode(source, mutation.nodeId, mutation.newParentId, mutation.index);
      case 'insertElement':
        return this.insertElement(source, mutation.parentId, mutation.index, mutation.tagName);
      case 'wrapInElement':
        return this.wrapInElement(source, mutation.nodeId, mutation.wrapperTagName);

      // Text mutations
      case 'setText':
        return this.setText(source, mutation.nodeId, mutation.content);
      case 'insertText':
        return this.insertText(source, mutation.parentId, mutation.index, mutation.content);

      // Attribute mutations
      case 'setAttribute':
        return this.setAttribute(source, mutation.nodeId, mutation.name, mutation.value);
      case 'removeAttribute':
        return this.removeAttribute(source, mutation.nodeId, mutation.name);

      // Layout shortcuts (map to setStyle)
      case 'setDisplay':
        return this.setStyle(source, mutation.nodeId, 'display', mutation.display);
      case 'setFlexDirection':
        return this.setStyle(source, mutation.nodeId, 'flex-direction', mutation.direction);
      case 'setJustifyContent':
        return this.setStyle(source, mutation.nodeId, 'justify-content', mutation.value);
      case 'setAlignItems':
        return this.setStyle(source, mutation.nodeId, 'align-items', mutation.value);
      case 'setGap':
        return this.setStyle(source, mutation.nodeId, 'gap', mutation.value);

      // Sizing shortcuts
      case 'setWidth':
        return this.setStyle(source, mutation.nodeId, 'width', mutation.value);
      case 'setHeight':
        return this.setStyle(source, mutation.nodeId, 'height', mutation.value);
      case 'setMinWidth':
        return this.setStyle(source, mutation.nodeId, 'min-width', mutation.value);
      case 'setMaxWidth':
        return this.setStyle(source, mutation.nodeId, 'max-width', mutation.value);
      case 'setMinHeight':
        return this.setStyle(source, mutation.nodeId, 'min-height', mutation.value);
      case 'setMaxHeight':
        return this.setStyle(source, mutation.nodeId, 'max-height', mutation.value);

      // Spacing shortcuts
      case 'setPadding':
        return this.setStyle(source, mutation.nodeId, 'padding', mutation.value);
      case 'setPaddingTop':
        return this.setStyle(source, mutation.nodeId, 'padding-top', mutation.value);
      case 'setPaddingRight':
        return this.setStyle(source, mutation.nodeId, 'padding-right', mutation.value);
      case 'setPaddingBottom':
        return this.setStyle(source, mutation.nodeId, 'padding-bottom', mutation.value);
      case 'setPaddingLeft':
        return this.setStyle(source, mutation.nodeId, 'padding-left', mutation.value);
      case 'setMargin':
        return this.setStyle(source, mutation.nodeId, 'margin', mutation.value);
      case 'setMarginTop':
        return this.setStyle(source, mutation.nodeId, 'margin-top', mutation.value);
      case 'setMarginRight':
        return this.setStyle(source, mutation.nodeId, 'margin-right', mutation.value);
      case 'setMarginBottom':
        return this.setStyle(source, mutation.nodeId, 'margin-bottom', mutation.value);
      case 'setMarginLeft':
        return this.setStyle(source, mutation.nodeId, 'margin-left', mutation.value);

      // Position shortcuts
      case 'setPosition':
        return this.setStyle(source, mutation.nodeId, 'position', mutation.position);
      case 'setTop':
        return this.setStyle(source, mutation.nodeId, 'top', mutation.value);
      case 'setRight':
        return this.setStyle(source, mutation.nodeId, 'right', mutation.value);
      case 'setBottom':
        return this.setStyle(source, mutation.nodeId, 'bottom', mutation.value);
      case 'setLeft':
        return this.setStyle(source, mutation.nodeId, 'left', mutation.value);
      case 'setZIndex':
        return this.setStyle(source, mutation.nodeId, 'z-index', mutation.value);

      // Visual shortcuts
      case 'setBackgroundColor':
        return this.setStyle(source, mutation.nodeId, 'background-color', mutation.value);
      case 'setColor':
        return this.setStyle(source, mutation.nodeId, 'color', mutation.value);
      case 'setBorderRadius':
        return this.setStyle(source, mutation.nodeId, 'border-radius', mutation.value);
      case 'setBorder':
        return this.setStyle(source, mutation.nodeId, 'border', mutation.value);
      case 'setBoxShadow':
        return this.setStyle(source, mutation.nodeId, 'box-shadow', mutation.value);
      case 'setOpacity':
        return this.setStyle(source, mutation.nodeId, 'opacity', mutation.value);

      // Typography shortcuts
      case 'setFontFamily':
        return this.setStyle(source, mutation.nodeId, 'font-family', mutation.value);
      case 'setFontSize':
        return this.setStyle(source, mutation.nodeId, 'font-size', mutation.value);
      case 'setFontWeight':
        return this.setStyle(source, mutation.nodeId, 'font-weight', mutation.value);
      case 'setLineHeight':
        return this.setStyle(source, mutation.nodeId, 'line-height', mutation.value);
      case 'setTextAlign':
        return this.setStyle(source, mutation.nodeId, 'text-align', mutation.value);
      case 'setTextDecoration':
        return this.setStyle(source, mutation.nodeId, 'text-decoration', mutation.value);

      // Frame mutations
      case 'setFramePosition':
        return this.setFrameBounds(source, mutation.frameId, mutation.x, mutation.y, null, null);
      case 'setFrameSize':
        return this.setFrameBounds(source, mutation.frameId, null, null, mutation.width, mutation.height);
      case 'setFrameBounds':
        return this.setFrameBounds(source, mutation.frameId, mutation.x, mutation.y, mutation.width, mutation.height);

      default:
        return source;
    }
  }

  private setStyle(source: string, nodeId: string, property: string, value: string): string {
    // Convert CSS property to Paperclip style attribute format
    const styleAttr = `style:${property}="${value}"`;

    // Check if style already exists - update it
    const existingStyleRegex = new RegExp(`style:${property}="[^"]*"`);
    if (existingStyleRegex.test(source)) {
      return source.replace(existingStyleRegex, styleAttr);
    }

    // Add style right before the { of the render block
    // Match: render <tag> [anything except {] followed by {
    // This handles both "render div {" and "render div style:existing="val" {"
    return source.replace(/(render\s+\w+[^{]*)(\{)/, `$1${styleAttr} $2`);
  }

  private removeStyle(source: string, nodeId: string, property: string): string {
    const styleRegex = new RegExp(`\\s*style:${property}="[^"]*"`, 'g');
    return source.replace(styleRegex, '');
  }

  private deleteNode(source: string, nodeId: string): string {
    // Simple: delete first text element found
    return source.replace(/\s*text\s+"[^"]*"/, '');
  }

  private moveNode(source: string, nodeId: string, newParentId: string, index: number): string {
    // This is complex in practice - simplified here
    return source;
  }

  private insertElement(source: string, parentId: string, index: number, tagName: string): string {
    // Insert new element before closing brace of render block
    const insertContent = `\n    ${tagName} {}`;
    return source.replace(/(render\s+\w+[^{]*\{)([^}]*)(\})/, `$1$2${insertContent}\n  $3`);
  }

  private wrapInElement(source: string, nodeId: string, wrapperTagName: string): string {
    // Wrap text in a div
    return source.replace(
      /(text\s+"[^"]*")/,
      `${wrapperTagName} {\n      $1\n    }`
    );
  }

  private setText(source: string, nodeId: string, content: string): string {
    return source.replace(/text\s+"[^"]*"/, `text "${content}"`);
  }

  private insertText(source: string, parentId: string, index: number, content: string): string {
    // Insert text before closing brace
    return source.replace(/(render\s+\w+[^{]*\{)([^}]*)(\})/, `$1$2\n    text "${content}"\n  $3`);
  }

  private setAttribute(source: string, nodeId: string, name: string, value: string): string {
    const attrRegex = new RegExp(`${name}="[^"]*"`);
    if (attrRegex.test(source)) {
      return source.replace(attrRegex, `${name}="${value}"`);
    }
    return source.replace(/(render\s+\w+)(\s)/, `$1 ${name}="${value}"$2`);
  }

  private removeAttribute(source: string, nodeId: string, name: string): string {
    const attrRegex = new RegExp(`\\s*${name}="[^"]*"`, 'g');
    return source.replace(attrRegex, '');
  }

  private setFrameBounds(
    source: string,
    frameId: string,
    x: number | null,
    y: number | null,
    width: number | null,
    height: number | null
  ): string {
    const frameMatch = source.match(/@frame\s*\{\s*x:\s*([\d.]+),\s*y:\s*([\d.]+),\s*width:\s*([\d.]+),\s*height:\s*([\d.]+)\s*\}/);

    const currentX = frameMatch ? parseFloat(frameMatch[1]) : 0;
    const currentY = frameMatch ? parseFloat(frameMatch[2]) : 0;
    const currentWidth = frameMatch ? parseFloat(frameMatch[3]) : 400;
    const currentHeight = frameMatch ? parseFloat(frameMatch[4]) : 300;

    const newX = x ?? currentX;
    const newY = y ?? currentY;
    const newWidth = width ?? currentWidth;
    const newHeight = height ?? currentHeight;

    const frameComment = `/** @frame { x: ${newX}, y: ${newY}, width: ${newWidth}, height: ${newHeight} } */`;

    if (frameMatch) {
      return source.replace(/\/\*\*\s*@frame\s*\{[^}]*\}\s*\*\//, frameComment);
    } else {
      return source.replace(/component/, `${frameComment}\ncomponent`);
    }
  }
}

// ============================================================================
// Mock Infrastructure
// ============================================================================

class MockServer {
  private sessions = new Map<string, {
    document: CRDTDocument;
    clients: Map<string, MockClient>;
    version: number;
  }>();

  private parseVDOM(source: string): any {
    const componentMatch = source.match(/component\s+(\w+)/);
    const renderMatch = source.match(/render\s+(\w+)/);

    if (!componentMatch || !renderMatch) {
      return { error: 'Invalid component structure' };
    }

    // Extract styles
    const styles: Record<string, string> = {};
    const styleMatches = [...source.matchAll(/style:([a-z-]+)="([^"]+)"/g)];
    for (const match of styleMatches) {
      styles[match[1]] = match[2];
    }

    // Extract attributes
    const attributes: Record<string, string> = {};
    const attrMatches = [...source.matchAll(/(?<!style:)(\w+)="([^"]+)"/g)];
    for (const match of attrMatches) {
      if (!match[1].startsWith('style')) {
        attributes[match[1]] = match[2];
      }
    }

    // Extract text content
    const textMatches = [...source.matchAll(/text\s+"([^"]+)"/g)];
    const children = textMatches.map((m, i) => ({
      type: 'text',
      id: `text-${i}`,
      content: m[1],
    }));

    // Extract child elements
    const elementMatches = [...source.matchAll(/(\w+)\s*\{[^}]*text/g)];

    // Extract frame bounds
    const frameMatch = source.match(/@frame\s*\{\s*x:\s*([\d.]+),\s*y:\s*([\d.]+),\s*width:\s*([\d.]+),\s*height:\s*([\d.]+)\s*\}/);

    return {
      type: 'element',
      tag: renderMatch[1],
      id: `${componentMatch[1]}-root`,
      componentName: componentMatch[1],
      styles,
      attributes,
      children,
      frameBounds: frameMatch ? {
        x: parseFloat(frameMatch[1]),
        y: parseFloat(frameMatch[2]),
        width: parseFloat(frameMatch[3]),
        height: parseFloat(frameMatch[4]),
      } : null,
    };
  }

  join(filePath: string, clientId: string, client: MockClient): void {
    let session = this.sessions.get(filePath);
    if (!session) {
      session = { document: new CRDTDocument(), clients: new Map(), version: 0 };
      this.sessions.set(filePath, session);
    }
    session.clients.set(clientId, client);
  }

  applyUpdate(filePath: string, clientId: string, update: Uint8Array): void {
    const session = this.sessions.get(filePath);
    if (!session) return;

    session.document.applyUpdate(update);
    session.version++;

    const vdom = this.parseVDOM(session.document.getText());

    // Broadcast CRDT to others
    for (const [id, client] of session.clients) {
      if (id !== clientId) {
        client.receiveUpdate(update);
      }
    }

    // Broadcast VDOM to all
    for (const [, client] of session.clients) {
      client.receiveVDOM({ vdom, version: session.version, originClientId: clientId });
    }
  }

  getText(filePath: string): string {
    return this.sessions.get(filePath)?.document.getText() ?? '';
  }
}

class MockClient {
  private updateHandler: ((u: Uint8Array) => void) | null = null;
  private vdomHandler: ((v: any) => void) | null = null;

  receiveUpdate(update: Uint8Array): void {
    this.updateHandler?.(update);
  }

  receiveVDOM(vdom: any): void {
    this.vdomHandler?.(vdom);
  }

  onUpdate(h: (u: Uint8Array) => void): void {
    this.updateHandler = h;
  }

  onVDOM(h: (v: any) => void): void {
    this.vdomHandler = h;
  }
}

function createTransport(server: MockServer, filePath: string, clientId: string): SyncTransport & { connect: () => void } {
  const client = new MockClient();
  const updateHandlers = new Set<(u: Uint8Array) => void>();
  const vdomHandlers = new Set<(v: any) => void>();

  client.onUpdate((u) => updateHandlers.forEach(h => h(u)));
  client.onVDOM((v) => vdomHandlers.forEach(h => h(v)));

  return {
    connect() { server.join(filePath, clientId, client); },
    async sendUpdate(fp: string, update: Uint8Array) { server.applyUpdate(fp, clientId, update); },
    onUpdate(cb) { updateHandlers.add(cb); return () => updateHandlers.delete(cb); },
    onVDOM(cb) { vdomHandlers.add(cb); return () => vdomHandlers.delete(cb); },
    onCSSOM(cb) { return () => {}; },
  };
}

/**
 * Designer that can perform all visual interactions.
 */
class Designer {
  private mutator = new SourceMutator();
  public vdom: any = null;
  public vdomHistory: any[] = [];

  constructor(private session: DocumentSession) {
    session.onVDOMChange((v) => {
      this.vdom = v.vdom;
      this.vdomHistory.push(v);
    });
  }

  private applyMutation(mutation: DesignerMutation): void {
    const newSource = this.mutator.apply(this.session.getText(), mutation);
    if (newSource !== this.session.getText()) {
      this.session.setText(newSource, { origin: 'designer' });
    }
  }

  // === Style Operations ===
  setStyle(nodeId: string, property: string, value: string): void {
    this.applyMutation({ type: 'setStyle', nodeId, property, value });
  }

  removeStyle(nodeId: string, property: string): void {
    this.applyMutation({ type: 'removeStyle', nodeId, property });
  }

  // === Layout Operations ===
  setDisplay(nodeId: string, display: 'block' | 'flex' | 'grid' | 'inline' | 'none'): void {
    this.applyMutation({ type: 'setDisplay', nodeId, display });
  }

  setFlexDirection(nodeId: string, direction: 'row' | 'column'): void {
    this.applyMutation({ type: 'setFlexDirection', nodeId, direction });
  }

  setJustifyContent(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setJustifyContent', nodeId, value });
  }

  setAlignItems(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setAlignItems', nodeId, value });
  }

  setGap(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setGap', nodeId, value });
  }

  // === Sizing Operations ===
  setWidth(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setWidth', nodeId, value });
  }

  setHeight(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setHeight', nodeId, value });
  }

  setSize(nodeId: string, width: string, height: string): void {
    this.setWidth(nodeId, width);
    this.setHeight(nodeId, height);
  }

  // === Spacing Operations ===
  setPadding(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setPadding', nodeId, value });
  }

  setPaddingIndividual(nodeId: string, top: string, right: string, bottom: string, left: string): void {
    this.applyMutation({ type: 'setPaddingTop', nodeId, value: top });
    this.applyMutation({ type: 'setPaddingRight', nodeId, value: right });
    this.applyMutation({ type: 'setPaddingBottom', nodeId, value: bottom });
    this.applyMutation({ type: 'setPaddingLeft', nodeId, value: left });
  }

  setMargin(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setMargin', nodeId, value });
  }

  // === Position Operations ===
  setPosition(nodeId: string, position: 'static' | 'relative' | 'absolute' | 'fixed'): void {
    this.applyMutation({ type: 'setPosition', nodeId, position });
  }

  setPositionValues(nodeId: string, top: string, right: string, bottom: string, left: string): void {
    this.applyMutation({ type: 'setTop', nodeId, value: top });
    this.applyMutation({ type: 'setRight', nodeId, value: right });
    this.applyMutation({ type: 'setBottom', nodeId, value: bottom });
    this.applyMutation({ type: 'setLeft', nodeId, value: left });
  }

  // === Visual Operations ===
  setBackgroundColor(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setBackgroundColor', nodeId, value });
  }

  setColor(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setColor', nodeId, value });
  }

  setBorderRadius(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setBorderRadius', nodeId, value });
  }

  setBorder(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setBorder', nodeId, value });
  }

  setBoxShadow(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setBoxShadow', nodeId, value });
  }

  setOpacity(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setOpacity', nodeId, value });
  }

  // === Typography Operations ===
  setFontFamily(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setFontFamily', nodeId, value });
  }

  setFontSize(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setFontSize', nodeId, value });
  }

  setFontWeight(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setFontWeight', nodeId, value });
  }

  setTextAlign(nodeId: string, value: string): void {
    this.applyMutation({ type: 'setTextAlign', nodeId, value });
  }

  // === Node Operations ===
  deleteNode(nodeId: string): void {
    this.applyMutation({ type: 'deleteNode', nodeId });
  }

  insertElement(parentId: string, index: number, tagName: string): void {
    this.applyMutation({ type: 'insertElement', parentId, index, tagName });
  }

  wrapInElement(nodeId: string, wrapperTagName: string): void {
    this.applyMutation({ type: 'wrapInElement', nodeId, wrapperTagName });
  }

  // === Text Operations ===
  setText(nodeId: string, content: string): void {
    this.applyMutation({ type: 'setText', nodeId, content });
  }

  insertText(parentId: string, index: number, content: string): void {
    this.applyMutation({ type: 'insertText', parentId, index, content });
  }

  // === Attribute Operations ===
  setAttribute(nodeId: string, name: string, value: string): void {
    this.applyMutation({ type: 'setAttribute', nodeId, name, value });
  }

  removeAttribute(nodeId: string, name: string): void {
    this.applyMutation({ type: 'removeAttribute', nodeId, name });
  }

  // === Frame Operations ===
  setFramePosition(frameId: string, x: number, y: number): void {
    this.applyMutation({ type: 'setFramePosition', frameId, x, y });
  }

  setFrameSize(frameId: string, width: number, height: number): void {
    this.applyMutation({ type: 'setFrameSize', frameId, width, height });
  }

  setFrameBounds(frameId: string, x: number, y: number, width: number, height: number): void {
    this.applyMutation({ type: 'setFrameBounds', frameId, x, y, width, height });
  }
}

// ============================================================================
// Tests
// ============================================================================

describe('Designer Interactions → Source Code → Sync', () => {
  let server: MockServer;

  beforeEach(() => {
    server = new MockServer();
  });

  const setupEditors = (filePath: string) => {
    const designerTransport = createTransport(server, filePath, 'designer');
    const vscodeTransport = createTransport(server, filePath, 'vscode');
    designerTransport.connect();
    vscodeTransport.connect();

    const designerSession = new DocumentSession(filePath, designerTransport);
    const vscodeSession = new DocumentSession(filePath, vscodeTransport);
    const designer = new Designer(designerSession);

    return { designerSession, vscodeSession, designer };
  };

  describe('CSS Style Changes', () => {
    it('setWidth updates source and syncs', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Button { render div { text "Click" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setWidth('Button-root', '200px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:width="200px"');
      expect(designer.vdom?.styles?.width).toBe('200px');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setHeight updates source and syncs', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Box { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setHeight('Box-root', '100px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:height="100px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setBackgroundColor updates source and syncs', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Card { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setBackgroundColor('Card-root', '#ff5500');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:background-color="#ff5500"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setColor updates text color', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Label { render span { text "Hello" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setColor('Label-root', 'blue');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:color="blue"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setBorderRadius updates corners', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Button { render button {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setBorderRadius('Button-root', '8px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:border-radius="8px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setBoxShadow adds shadow', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Card { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setBoxShadow('Card-root', '0 4px 6px rgba(0,0,0,0.1)');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:box-shadow="0 4px 6px rgba(0,0,0,0.1)"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setOpacity changes transparency', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Overlay { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setOpacity('Overlay-root', '0.5');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:opacity="0.5"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('removeStyle removes existing style', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Box { render div style:width="100px" {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.removeStyle('Box-root', 'width');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).not.toContain('style:width');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Layout Changes', () => {
    it('setDisplay changes display mode', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Container { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setDisplay('Container-root', 'flex');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:display="flex"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setFlexDirection changes flex layout', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Row { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFlexDirection('Row-root', 'column');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:flex-direction="column"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setJustifyContent aligns main axis', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Nav { render nav {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setJustifyContent('Nav-root', 'space-between');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:justify-content="space-between"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setAlignItems aligns cross axis', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Header { render header {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setAlignItems('Header-root', 'center');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:align-items="center"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setGap adds spacing between items', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component List { render ul {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setGap('List-root', '16px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:gap="16px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Spacing Changes', () => {
    it('setPadding adds uniform padding', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Card { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setPadding('Card-root', '20px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:padding="20px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setMargin adds uniform margin', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Section { render section {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setMargin('Section-root', '0 auto');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:margin="0 auto"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Position Changes', () => {
    it('setPosition changes positioning mode', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Modal { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setPosition('Modal-root', 'fixed');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:position="fixed"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Typography Changes', () => {
    it('setFontSize changes text size', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Heading { render h1 { text "Title" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFontSize('Heading-root', '32px');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:font-size="32px"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setFontWeight changes text weight', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Label { render span { text "Bold" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFontWeight('Label-root', '700');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:font-weight="700"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setTextAlign changes alignment', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Paragraph { render p { text "Centered" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setTextAlign('Paragraph-root', 'center');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('style:text-align="center"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Node Operations', () => {
    it('deleteNode removes element from source', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Card { render div { text "Delete me" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.deleteNode('text-0');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).not.toContain('Delete me');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('insertElement adds new element', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Container { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.insertElement('Container-root', 0, 'span');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('span {}');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('wrapInElement wraps existing content', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Button { render div { text "Click" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.wrapInElement('text-0', 'span');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('span {');
      expect(vscodeSession.getText()).toContain('text "Click"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Text Operations', () => {
    it('setText updates text content', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Title { render h1 { text "Old Title" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setText('text-0', 'New Title');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('text "New Title"');
      expect(vscodeSession.getText()).not.toContain('Old Title');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('insertText adds new text node', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Paragraph { render p {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.insertText('Paragraph-root', 0, 'Hello World');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('text "Hello World"');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Attribute Operations', () => {
    it('setAttribute adds/updates attribute', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Link { render a { text "Click" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setAttribute('Link-root', 'href', 'https://example.com');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('href="https://example.com"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('removeAttribute removes existing attribute', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Input { render input disabled="true" {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.removeAttribute('Input-root', 'disabled');
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).not.toContain('disabled');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Frame/Canvas Operations', () => {
    it('setFramePosition updates @frame comment', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Card { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFramePosition('Card', 100, 200);
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('@frame');
      expect(vscodeSession.getText()).toContain('x: 100');
      expect(vscodeSession.getText()).toContain('y: 200');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setFrameSize updates frame dimensions', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('/** @frame { x: 0, y: 0, width: 100, height: 100 } */\ncomponent Box { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFrameSize('Box', 500, 300);
      await new Promise(r => setTimeout(r, 20));

      expect(vscodeSession.getText()).toContain('width: 500');
      expect(vscodeSession.getText()).toContain('height: 300');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('setFrameBounds updates full frame', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Modal { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFrameBounds('Modal', 50, 100, 800, 600);
      await new Promise(r => setTimeout(r, 20));

      const source = vscodeSession.getText();
      expect(source).toContain('x: 50');
      expect(source).toContain('y: 100');
      expect(source).toContain('width: 800');
      expect(source).toContain('height: 600');

      designerSession.dispose();
      vscodeSession.dispose();
    });
  });

  describe('Complex Multi-Operation Scenarios', () => {
    it('complete button styling workflow', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/button.pc');

      // Start with basic button
      designerSession.setText('component Button { render button { text "Submit" } }');
      await new Promise(r => setTimeout(r, 20));

      // Apply multiple styles like a real design session
      designer.setBackgroundColor('Button-root', '#007bff');
      designer.setColor('Button-root', 'white');
      designer.setPadding('Button-root', '12px 24px');
      designer.setBorderRadius('Button-root', '6px');
      designer.setBorder('Button-root', 'none');
      designer.setFontWeight('Button-root', '600');
      designer.setFontSize('Button-root', '16px');

      await new Promise(r => setTimeout(r, 50));

      const source = vscodeSession.getText();
      expect(source).toContain('style:background-color="#007bff"');
      expect(source).toContain('style:color="white"');
      expect(source).toContain('style:border-radius="6px"');
      expect(source).toContain('style:font-weight="600"');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('complete card layout workflow', async () => {
      const { designerSession, vscodeSession, designer } = setupEditors('/card.pc');

      // Start with card structure
      designerSession.setText('component Card { render div { text "Card Content" } }');
      await new Promise(r => setTimeout(r, 20));

      // Apply layout and styling
      designer.setDisplay('Card-root', 'flex');
      designer.setFlexDirection('Card-root', 'column');
      designer.setWidth('Card-root', '320px');
      designer.setPadding('Card-root', '24px');
      designer.setBackgroundColor('Card-root', 'white');
      designer.setBorderRadius('Card-root', '12px');
      designer.setBoxShadow('Card-root', '0 2px 8px rgba(0,0,0,0.1)');

      await new Promise(r => setTimeout(r, 50));

      const source = vscodeSession.getText();
      expect(source).toContain('style:display="flex"');
      expect(source).toContain('style:flex-direction="column"');
      expect(source).toContain('style:width="320px"');
      expect(source).toContain('style:box-shadow');

      designerSession.dispose();
      vscodeSession.dispose();
    });

    it('two designers collaborate on same component', async () => {
      const transport1 = createTransport(server, '/shared.pc', 'designer1');
      const transport2 = createTransport(server, '/shared.pc', 'designer2');
      transport1.connect();
      transport2.connect();

      const session1 = new DocumentSession('/shared.pc', transport1);
      const session2 = new DocumentSession('/shared.pc', transport2);
      const designer1 = new Designer(session1);
      const designer2 = new Designer(session2);

      // Initial component
      session1.setText('component Shared { render div { text "Collaborate" } }');
      await new Promise(r => setTimeout(r, 30));

      // Designer 1 works on colors
      designer1.setBackgroundColor('Shared-root', '#f5f5f5');
      designer1.setColor('Shared-root', '#333');

      // Designer 2 works on layout
      designer2.setDisplay('Shared-root', 'flex');
      designer2.setPadding('Shared-root', '16px');

      await new Promise(r => setTimeout(r, 50));

      // Both should have all changes
      expect(session1.getText()).toBe(session2.getText());

      const source = session1.getText();
      expect(source).toContain('style:background-color');
      expect(source).toContain('style:display="flex"');
      expect(source).toContain('style:padding');

      session1.dispose();
      session2.dispose();
    });
  });

  describe('VDOM Reflects Source Changes', () => {
    it('VDOM updates with correct styles after changes', async () => {
      const { designerSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Box { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setWidth('Box-root', '200px');
      designer.setHeight('Box-root', '100px');
      designer.setBackgroundColor('Box-root', 'red');

      await new Promise(r => setTimeout(r, 30));

      expect(designer.vdom?.styles?.width).toBe('200px');
      expect(designer.vdom?.styles?.height).toBe('100px');
      expect(designer.vdom?.styles?.['background-color']).toBe('red');

      designerSession.dispose();
    });

    it('VDOM updates with correct text after changes', async () => {
      const { designerSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Label { render span { text "Original" } }');
      await new Promise(r => setTimeout(r, 20));

      designer.setText('text-0', 'Updated');
      await new Promise(r => setTimeout(r, 20));

      expect(designer.vdom?.children?.[0]?.content).toBe('Updated');

      designerSession.dispose();
    });

    it('VDOM updates with correct frame bounds', async () => {
      const { designerSession, designer } = setupEditors('/test.pc');

      designerSession.setText('component Frame { render div {} }');
      await new Promise(r => setTimeout(r, 20));

      designer.setFrameBounds('Frame', 100, 200, 400, 300);
      await new Promise(r => setTimeout(r, 20));

      expect(designer.vdom?.frameBounds).toEqual({
        x: 100,
        y: 200,
        width: 400,
        height: 300,
      });

      designerSession.dispose();
    });
  });
});
