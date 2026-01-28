/**
 * Example React component for testing hybrid rendering
 */

import React, { useState } from "react";

export interface DatePickerProps {
  label?: string;
  initialDate?: string;
  onChange?: (date: string) => void;
}

export function DatePicker({
  label = "Select Date",
  initialDate = "",
  onChange,
}: DatePickerProps) {
  const [date, setDate] = useState(initialDate);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newDate = e.target.value;
    setDate(newDate);
    onChange?.(newDate);
  };

  return (
    <div style={{ padding: "1rem", border: "1px solid #ccc", borderRadius: "4px" }}>
      <label style={{ display: "block", marginBottom: "0.5rem", fontWeight: "bold" }}>
        {label}
      </label>
      <input
        type="date"
        value={date}
        onChange={handleChange}
        style={{
          padding: "0.5rem",
          border: "1px solid #ddd",
          borderRadius: "4px",
          fontSize: "1rem",
        }}
      />
      {date && (
        <div style={{ marginTop: "0.5rem", fontSize: "0.875rem", color: "#666" }}>
          Selected: {date}
        </div>
      )}
    </div>
  );
}
