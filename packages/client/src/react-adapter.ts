/**
 * React Adapter - integrates React components with the patch applier
 */

import { createRoot, Root } from "react-dom/client";
import { createElement } from "react";
import type { ComponentMetadata } from "./component-registry";

export interface ReactComponentMount {
  container: HTMLElement;
  root: Root;
  metadata: ComponentMetadata;
  props: any;
}

export interface ReactAdapter {
  mount(
    container: HTMLElement,
    metadata: ComponentMetadata,
    props: any
  ): ReactComponentMount;
  update(mount: ReactComponentMount, props: any): void;
  unmount(mount: ReactComponentMount): void;
}

export function createReactAdapter(): ReactAdapter {
  return {
    mount(container, metadata, props) {
      const root = createRoot(container);
      const Component = metadata.module[metadata.exportName];

      if (!Component) {
        throw new Error(
          `Component ${metadata.exportName} not found in module`
        );
      }

      root.render(createElement(Component, props));

      return {
        container,
        root,
        metadata,
        props,
      };
    },

    update(mount, props) {
      const Component = mount.metadata.module[mount.metadata.exportName];
      mount.root.render(createElement(Component, props));
      mount.props = props;
    },

    unmount(mount) {
      mount.root.unmount();
    },
  };
}
