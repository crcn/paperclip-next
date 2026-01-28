/**
 * Component Registry - tracks loaded React components
 */

export interface ComponentMetadata {
  id: string;
  framework: "react";
  // The ES module object containing the component
  // Example: import * as DatePickerModule from './DatePicker'
  // Then module = DatePickerModule, which has { DatePicker: Component }
  // Access via: module[exportName] or module.DatePicker
  module: any;
  exportName: string;
}

export interface ComponentRegistry {
  register(metadata: ComponentMetadata): void;
  get(id: string): ComponentMetadata | undefined;
  has(id: string): boolean;
  list(): ComponentMetadata[];
}

export function createComponentRegistry(): ComponentRegistry {
  const components = new Map<string, ComponentMetadata>();

  return {
    register(metadata: ComponentMetadata) {
      components.set(metadata.id, metadata);
    },

    get(id: string) {
      return components.get(id);
    },

    has(id: string) {
      return components.has(id);
    },

    list() {
      return Array.from(components.values());
    },
  };
}
