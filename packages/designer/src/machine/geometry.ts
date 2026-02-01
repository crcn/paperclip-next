/**
 * Geometry utilities for canvas transforms
 */

import { Box, Point, Transform } from "./state";

export const MIN_ZOOM = 0.01;
export const MAX_ZOOM = 64;
export const ZOOM_SENSITIVITY = 250;
export const INITIAL_ZOOM_PADDING = 100;

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function screenToCanvas(
  point: Point,
  transform: Transform,
  scroll: Point = { x: 0, y: 0 }
): Point {
  return {
    x: (point.x - transform.x) / transform.z + scroll.x,
    y: (point.y - transform.y) / transform.z + scroll.y,
  };
}

export function canvasToScreen(
  point: Point,
  transform: Transform,
  scroll: Point = { x: 0, y: 0 }
): Point {
  return {
    x: (point.x - scroll.x) * transform.z + transform.x,
    y: (point.y - scroll.y) * transform.z + transform.y,
  };
}

export function getFramesBounds(frames: Box[]): Box | null {
  if (frames.length === 0) return null;

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const frame of frames) {
    minX = Math.min(minX, frame.x);
    minY = Math.min(minY, frame.y);
    maxX = Math.max(maxX, frame.x + frame.width);
    maxY = Math.max(maxY, frame.y + frame.height);
  }

  return {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

export function centerTransformOnBounds(
  viewportSize: { width: number; height: number },
  bounds: Box,
  zoomToFit: boolean = true
): Transform {
  const scaleX = (viewportSize.width - INITIAL_ZOOM_PADDING * 2) / bounds.width;
  const scaleY = (viewportSize.height - INITIAL_ZOOM_PADDING * 2) / bounds.height;
  const scale = zoomToFit ? Math.min(scaleX, scaleY, 1) : 1;

  const centerX = bounds.x + bounds.width / 2;
  const centerY = bounds.y + bounds.height / 2;

  return {
    x: viewportSize.width / 2 - centerX * scale,
    y: viewportSize.height / 2 - centerY * scale,
    z: scale,
  };
}

export function centerTransformZoom(
  currentTransform: Transform,
  viewportSize: { width: number; height: number },
  newZoom: number,
  centerPoint?: Point
): Transform {
  const oldZoom = currentTransform.z;
  const clampedZoom = clamp(newZoom, MIN_ZOOM, MAX_ZOOM);

  const center = centerPoint ?? {
    x: viewportSize.width / 2,
    y: viewportSize.height / 2,
  };

  const canvasX = (center.x - currentTransform.x) / oldZoom;
  const canvasY = (center.y - currentTransform.y) / oldZoom;

  const newX = center.x - canvasX * clampedZoom;
  const newY = center.y - canvasY * clampedZoom;

  return {
    x: newX,
    y: newY,
    z: clampedZoom,
  };
}
