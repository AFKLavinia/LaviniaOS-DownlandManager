import React from "react";
import type { SvgIconProps } from "./SvgIconProps";

export const SpeedIcon: React.FC<SvgIconProps> = ({
  size = 18,
  className = "",
  strokeWidth = 2,
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={strokeWidth}
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
    aria-hidden="true"
  >
    <path d="M12 2v4" />
    <path d="m4.93 4.93 2.83 2.83" />
    <path d="M2 12h4" />
    <path d="m4.93 19.07 2.83-2.83" />
    <path d="M14 12a2 2 0 1 0-4 0 2 2 0 0 0 4 0Z" />
    <path d="m14 12 4-4" />
    <path d="M12 22a10 10 0 0 0 10-10c0-4.42-2.87-8.17-6.84-9.5" />
  </svg>
);
