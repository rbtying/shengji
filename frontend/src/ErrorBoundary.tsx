import * as React from "react";

import type { JSX } from "react";

interface IProps {
  fallback: React.ReactNode;
  children: React.ReactNode;
}

interface IState {
  hasError: boolean;
}

/** Shows `fallback` instead of a blank page if something below it throws while rendering. */
class ErrorBoundary extends React.Component<IProps, IState> {
  state: IState = { hasError: false };

  static getDerivedStateFromError(): IState {
    return { hasError: true };
  }

  componentDidCatch(error: unknown): void {
    console.error("Unexpected error while rendering:", error);
  }

  render(): JSX.Element {
    return (
      <>{this.state.hasError ? this.props.fallback : this.props.children}</>
    );
  }
}

export default ErrorBoundary;
