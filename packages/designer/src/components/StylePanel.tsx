"use client";

import React, { useCallback, useMemo } from "react";
import { useDispatch } from "@paperclip/common";
import { DesignerMachine } from "../machine";
import type { DesignerEvent } from "../machine/state";
import { StylePropertyRow } from "./StylePropertyRow";

const styles = {
  panel: {
    display: "flex",
    flexDirection: "column" as const,
    height: "100%",
    backgroundColor: "#1e1e1e",
    color: "#e0e0e0",
    fontSize: "12px",
    fontFamily: "system-ui, -apple-system, sans-serif",
  },
  empty: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    height: "100%",
    color: "#666",
  },
  placeholder: {
    textAlign: "center" as const,
    padding: "20px",
  },
  section: {
    padding: "8px 12px",
  },
  sectionTitle: {
    fontSize: "11px",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    color: "#888",
    marginBottom: "8px",
    letterSpacing: "0.5px",
  },
  properties: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "2px",
  },
};

/**
 * Style Panel - displays and edits CSS properties for the selected element.
 *
 * Phase 1 (Simplified):
 * - Display inline styles only
 * - Edit existing properties
 * - Add new properties
 * - Remove properties
 *
 * Phase 2 (Deferred):
 * - Variant selector
 * - Mixin/extends selector
 * - Token autocomplete
 * - Color picker
 * - Computed style origins
 */
export function StylePanel() {
  const dispatch = useDispatch<DesignerEvent>();

  const selectedElement = DesignerMachine.useSelector((s) => s.selectedElement);
  const computedStyles = DesignerMachine.useSelector((s) => s.computedStyles);

  // Memoize sorted properties to avoid re-sorting on every render
  const sortedProperties = useMemo(() => {
    return Object.entries(computedStyles).sort(([a], [b]) => a.localeCompare(b));
  }, [computedStyles]);

  const handleStyleChange = useCallback(
    (property: string, value: string) => {
      dispatch({
        type: "style/changed",
        payload: { property, value },
      });
    },
    [dispatch]
  );

  const handleStyleRemove = useCallback(
    (property: string) => {
      dispatch({
        type: "style/removed",
        payload: { property },
      });
    },
    [dispatch]
  );

  if (!selectedElement) {
    return (
      <div style={{ ...styles.panel, ...styles.empty }}>
        <div style={styles.placeholder}>Select an element to edit styles</div>
      </div>
    );
  }

  return (
    <div style={styles.panel}>
      {/* Phase 2: Variant Selector will go here */}
      {/* Phase 2: Mixin Selector will go here */}

      {/* Properties Section */}
      <section style={styles.section}>
        <h3 style={styles.sectionTitle}>Styles</h3>
        <div style={styles.properties}>
          {sortedProperties.map(([property, styleValue]) => (
            <StylePropertyRow
              key={property}
              property={property}
              value={styleValue.value}
              origin={styleValue.origin}
              onChange={(value) => handleStyleChange(property, value)}
              onRemove={() => handleStyleRemove(property)}
            />
          ))}

          {/* Add new property row */}
          <StylePropertyRow
            property=""
            value=""
            isNew
            onChange={handleStyleChange}
          />
        </div>
      </section>
    </div>
  );
}
