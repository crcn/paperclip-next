"use client";

import React, { useCallback } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine } from "../machine";
import type { DesignerEvent, VNode, VDocument } from "../machine/state";

const styles = {
  panel: {
    display: "flex",
    flexDirection: "column" as const,
    height: "100%",
    backgroundColor: "#1e1e1e",
    color: "#e0e0e0",
    fontSize: "12px",
    fontFamily: "system-ui, -apple-system, sans-serif",
    overflow: "auto",
  },
  header: {
    padding: "8px 12px",
    borderBottom: "1px solid #333",
    fontWeight: 600,
    fontSize: "11px",
    textTransform: "uppercase" as const,
    color: "#888",
    letterSpacing: "0.5px",
  },
  tree: {
    padding: "4px 0",
  },
  empty: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    height: "100%",
    color: "#666",
  },
};

interface LayerNodeProps {
  node: VNode;
  depth: number;
  path: number[];
  frameIndex: number;
  expandedNodes: Set<string>;
  selectedNodeId?: string;
  onToggle: (nodeId: string) => void;
  onClick: (nodeId: string, sourceId: string, frameIndex: number) => void;
}

const nodeStyles = {
  row: {
    display: "flex",
    alignItems: "center",
    padding: "3px 8px",
    cursor: "pointer",
    borderRadius: "3px",
    marginLeft: "4px",
    marginRight: "4px",
  },
  rowHover: {
    backgroundColor: "rgba(255, 255, 255, 0.05)",
  },
  rowSelected: {
    backgroundColor: "rgba(74, 144, 226, 0.3)",
  },
  toggle: {
    width: "16px",
    height: "16px",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    color: "#666",
    fontSize: "10px",
    flexShrink: 0,
  },
  tag: {
    color: "#569cd6",
  },
  text: {
    color: "#ce9178",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
    maxWidth: "150px",
  },
  error: {
    color: "#f44747",
  },
  comment: {
    color: "#6a9955",
    fontStyle: "italic" as const,
  },
  nodeIcon: {
    marginRight: "4px",
    color: "#888",
    fontSize: "10px",
  },
};

function LayerNode({
  node,
  depth,
  path,
  frameIndex,
  expandedNodes,
  selectedNodeId,
  onToggle,
  onClick,
}: LayerNodeProps) {
  const [isHovered, setIsHovered] = React.useState(false);

  // Extract node info based on oneof type
  const element = node.element;
  const text = node.text;
  const comment = node.comment;
  const error = node.error;

  // Get node ID (semantic_id or generate one)
  const nodeId = element?.semanticId ?? `node-${path.join("-")}`;
  const sourceId = element?.sourceId ?? nodeId;
  const hasChildren = element?.children && element.children.length > 0;
  const isExpanded = expandedNodes.has(nodeId);
  const isSelected = selectedNodeId === nodeId;

  const handleToggle = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (hasChildren) {
        onToggle(nodeId);
      }
    },
    [hasChildren, nodeId, onToggle]
  );

  const handleClick = useCallback(() => {
    onClick(nodeId, sourceId, frameIndex);
  }, [nodeId, sourceId, frameIndex, onClick]);

  const rowStyle = {
    ...nodeStyles.row,
    paddingLeft: `${8 + depth * 16}px`,
    ...(isHovered ? nodeStyles.rowHover : {}),
    ...(isSelected ? nodeStyles.rowSelected : {}),
  };

  // Render different node types
  if (element) {
    return (
      <>
        <div
          style={rowStyle}
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
          onClick={handleClick}
        >
          <span style={nodeStyles.toggle} onClick={handleToggle}>
            {hasChildren ? (isExpanded ? "▼" : "▶") : ""}
          </span>
          <span style={nodeStyles.tag}>&lt;{element.tag}&gt;</span>
        </div>
        {isExpanded &&
          element.children?.map((child, index) => (
            <LayerNode
              key={`${nodeId}-${index}`}
              node={child}
              depth={depth + 1}
              path={[...path, index]}
              frameIndex={frameIndex}
              expandedNodes={expandedNodes}
              selectedNodeId={selectedNodeId}
              onToggle={onToggle}
              onClick={onClick}
            />
          ))}
      </>
    );
  }

  if (text) {
    // Skip empty or whitespace-only text nodes
    const content = text.content?.trim();
    if (!content) return null;

    return (
      <div
        style={rowStyle}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <span style={nodeStyles.toggle} />
        <span style={nodeStyles.text}>"{content}"</span>
      </div>
    );
  }

  if (comment) {
    return (
      <div
        style={rowStyle}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <span style={nodeStyles.toggle} />
        <span style={nodeStyles.comment}>&lt;!-- {comment.content} --&gt;</span>
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={rowStyle}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <span style={nodeStyles.toggle} />
        <span style={nodeStyles.error}>Error: {error.message}</span>
      </div>
    );
  }

  return null;
}

interface FrameTreeProps {
  frameIndex: number;
  rootNode: VNode;
  expandedNodes: Set<string>;
  selectedNodeId?: string;
  onToggle: (nodeId: string) => void;
  onClick: (nodeId: string, sourceId: string, frameIndex: number) => void;
}

const frameStyles = {
  frame: {
    marginBottom: "8px",
  },
  frameHeader: {
    display: "flex",
    alignItems: "center",
    padding: "6px 12px",
    backgroundColor: "rgba(255, 255, 255, 0.03)",
    borderBottom: "1px solid #333",
    cursor: "pointer",
  },
  frameHeaderHover: {
    backgroundColor: "rgba(255, 255, 255, 0.06)",
  },
  frameName: {
    fontWeight: 500,
    color: "#dcdcaa",
  },
  frameIndex: {
    marginLeft: "auto",
    color: "#666",
    fontSize: "10px",
  },
};

function FrameTree({
  frameIndex,
  rootNode,
  expandedNodes,
  selectedNodeId,
  onToggle,
  onClick,
}: FrameTreeProps) {
  const [isHeaderHovered, setIsHeaderHovered] = React.useState(false);
  const element = rootNode.element;
  const frameName = element?.attributes?.["data-frame-name"] ?? `Frame ${frameIndex + 1}`;

  const headerStyle = {
    ...frameStyles.frameHeader,
    ...(isHeaderHovered ? frameStyles.frameHeaderHover : {}),
  };

  return (
    <div style={frameStyles.frame}>
      <div
        style={headerStyle}
        onMouseEnter={() => setIsHeaderHovered(true)}
        onMouseLeave={() => setIsHeaderHovered(false)}
      >
        <span style={frameStyles.frameName}>{frameName}</span>
        <span style={frameStyles.frameIndex}>#{frameIndex + 1}</span>
      </div>
      <LayerNode
        node={rootNode}
        depth={0}
        path={[frameIndex]}
        frameIndex={frameIndex}
        expandedNodes={expandedNodes}
        selectedNodeId={selectedNodeId}
        onToggle={onToggle}
        onClick={onClick}
      />
    </div>
  );
}

/**
 * Layer Panel - displays a tree view of the VDOM structure.
 *
 * Allows users to:
 * - Navigate the document structure
 * - Select elements for style editing
 * - Expand/collapse nested elements
 */
export function LayerPanel() {
  const dispatch = useDispatch<DesignerEvent>();

  const document = DesignerMachine.useSelector((s) => s.document);
  const expandedNodes = DesignerMachine.useSelector((s) => s.layerPanel.expandedNodes);
  const selectedElement = DesignerMachine.useSelector((s) => s.selectedElement);

  const handleToggle = useCallback(
    (nodeId: string) => {
      if (expandedNodes.has(nodeId)) {
        dispatch({ type: "layer/nodeCollapsed", payload: { nodeId } });
      } else {
        dispatch({ type: "layer/nodeExpanded", payload: { nodeId } });
      }
    },
    [dispatch, expandedNodes]
  );

  const handleClick = useCallback(
    (nodeId: string, sourceId: string, frameIndex: number) => {
      dispatch({
        type: "layer/nodeClicked",
        payload: { nodeId, sourceId, frameIndex },
      });
    },
    [dispatch]
  );

  if (!document || document.nodes.length === 0) {
    return (
      <div style={{ ...styles.panel, ...styles.empty }}>
        <span>No document loaded</span>
      </div>
    );
  }

  return (
    <div style={styles.panel}>
      <div style={styles.header}>Layers</div>
      <div style={styles.tree}>
        {document.nodes.map((node, index) => (
          <FrameTree
            key={index}
            frameIndex={index}
            rootNode={node}
            expandedNodes={expandedNodes}
            selectedNodeId={selectedElement?.nodeId}
            onToggle={handleToggle}
            onClick={handleClick}
          />
        ))}
      </div>
    </div>
  );
}
