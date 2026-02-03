"use client";

import React, { useState, useCallback, useRef, useEffect, useId } from "react";

interface StylePropertyRowProps {
  property: string;
  value: string;
  origin?: "inline" | "mixin" | "inherited" | "default";
  isNew?: boolean;
  /** For existing properties: (value) => void. For new properties: (property, value) => void */
  onChange: (propertyOrValue: string, value?: string) => void;
  onRemove?: () => void;
}

const baseStyles = {
  row: {
    display: "flex",
    alignItems: "center",
    padding: "4px 0",
    gap: "4px",
    borderRadius: "3px",
  },
  rowHover: {
    backgroundColor: "rgba(255, 255, 255, 0.05)",
  },
  property: {
    flex: "0 0 auto",
    minWidth: "80px",
    maxWidth: "120px",
  },
  propertyName: {
    color: "#9cdcfe",
    fontSize: "12px",
  },
  propertyInput: {
    background: "transparent",
    border: "none",
    outline: "none",
    color: "#9cdcfe",
    fontSize: "12px",
    width: "100%",
    padding: "2px 4px",
  },
  colon: {
    color: "#666",
    marginRight: "4px",
  },
  value: {
    flex: "1 1 auto",
  },
  valueInput: {
    background: "transparent",
    border: "none",
    outline: "none",
    color: "#ce9178",
    fontSize: "12px",
    width: "100%",
    padding: "2px 4px",
  },
  valueInputFocused: {
    backgroundColor: "rgba(255, 255, 255, 0.1)",
    borderRadius: "2px",
  },
  remove: {
    background: "transparent",
    border: "none",
    color: "#666",
    cursor: "pointer",
    padding: "2px 6px",
    fontSize: "14px",
    lineHeight: 1,
    borderRadius: "2px",
  },
  removeHover: {
    color: "#e74c3c",
    backgroundColor: "rgba(231, 76, 60, 0.1)",
  },
  origin: {
    fontSize: "10px",
    color: "#666",
    padding: "2px 4px",
    borderRadius: "2px",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
  },
  newRow: {
    opacity: 0.6,
  },
};

/**
 * A single row in the style panel for editing a CSS property.
 *
 * Phase 1 (Simplified):
 * - Property name input (for new properties)
 * - Value input
 * - Remove button (for inline styles)
 *
 * Phase 2 (Deferred):
 * - Token autocomplete
 * - Color picker for color properties
 * - Origin indicator with tooltip
 */
export function StylePropertyRow({
  property,
  value,
  origin = "inline",
  isNew = false,
  onChange,
  onRemove,
}: StylePropertyRowProps) {
  const [editingProperty, setEditingProperty] = useState(property);
  const [editingValue, setEditingValue] = useState(value);
  const [isHovered, setIsHovered] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const [isRemoveHovered, setIsRemoveHovered] = useState(false);
  const propertyInputRef = useRef<HTMLInputElement>(null);
  const valueInputRef = useRef<HTMLInputElement>(null);
  const inputId = useId();

  // Sync local state when prop changes (controlled component pattern)
  useEffect(() => {
    setEditingValue(value);
  }, [value]);

  useEffect(() => {
    setEditingProperty(property);
  }, [property]);

  const handleValueChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setEditingValue(e.target.value);
    },
    []
  );

  const handlePropertyChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setEditingProperty(e.target.value);
    },
    []
  );

  const handleBlur = useCallback(() => {
    setIsFocused(false);
    // Only trigger change if value actually changed
    if (isNew) {
      // For new properties, need both property and value
      if (editingProperty && editingValue) {
        onChange(editingProperty, editingValue);
        // Clear inputs for next entry
        setEditingProperty("");
        setEditingValue("");
      }
    } else {
      // For existing properties, only value changes
      if (editingValue !== value) {
        onChange(editingValue);
      }
    }
  }, [isNew, editingProperty, editingValue, property, value, onChange]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        if (isNew && !editingProperty && e.target === propertyInputRef.current) {
          // Focus value input after entering property name
          valueInputRef.current?.focus();
        } else {
          // Submit on Enter
          (e.target as HTMLInputElement).blur();
        }
      } else if (e.key === "Escape") {
        // Reset to original values
        setEditingValue(value);
        setEditingProperty(property);
        (e.target as HTMLInputElement).blur();
      } else if (e.key === "Tab" && isNew && e.target === propertyInputRef.current) {
        // Tab from property to value input
        e.preventDefault();
        valueInputRef.current?.focus();
      }
    },
    [value, property, isNew, editingProperty]
  );

  const rowStyle = {
    ...baseStyles.row,
    ...(isHovered ? baseStyles.rowHover : {}),
    ...(isNew ? baseStyles.newRow : {}),
  };

  const valueInputStyle = {
    ...baseStyles.valueInput,
    ...(isFocused ? baseStyles.valueInputFocused : {}),
  };

  const removeStyle = {
    ...baseStyles.remove,
    ...(isRemoveHovered ? baseStyles.removeHover : {}),
  };

  return (
    <div
      style={rowStyle}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <label htmlFor={inputId} style={baseStyles.property}>
        {isNew ? (
          <input
            ref={propertyInputRef}
            type="text"
            value={editingProperty}
            onChange={handlePropertyChange}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            placeholder="property"
            style={baseStyles.propertyInput}
            aria-label="CSS property name"
          />
        ) : (
          <span style={baseStyles.propertyName}>{property}</span>
        )}
      </label>

      <span style={baseStyles.colon}>:</span>

      <span style={baseStyles.value}>
        <input
          id={inputId}
          ref={valueInputRef}
          type="text"
          value={editingValue}
          onChange={handleValueChange}
          onFocus={() => setIsFocused(true)}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          placeholder={isNew ? "value" : undefined}
          style={valueInputStyle}
          aria-label={`Value for ${property || "new property"}`}
        />
      </span>

      {/* Remove button for inline styles */}
      {!isNew && origin === "inline" && onRemove && (
        <button
          style={removeStyle}
          onClick={onRemove}
          onMouseEnter={() => setIsRemoveHovered(true)}
          onMouseLeave={() => setIsRemoveHovered(false)}
          title="Remove property"
          aria-label={`Remove ${property}`}
          type="button"
        >
          &times;
        </button>
      )}

      {/* Origin indicator for non-inline styles */}
      {!isNew && origin !== "inline" && (
        <span style={baseStyles.origin} title={`From ${origin}`}>
          {origin === "mixin" ? "M" : origin === "inherited" ? "I" : "D"}
        </span>
      )}
    </div>
  );
}
