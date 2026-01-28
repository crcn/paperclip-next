/// <reference types="vite/client" />

// Paperclip module declarations
declare module '*.pc' {
  import { FC, ReactNode } from 'react';

  // Export all components as React functional components
  // You can be more specific based on your actual components
  export const Button: FC<{ children?: ReactNode }>;
  export const Card: FC<{ children?: ReactNode }>;
  export const Hero: FC<{ children?: ReactNode }>;
  export const Feature: FC<{ children?: ReactNode }>;
}
