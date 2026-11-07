import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "primary" | "danger";
  icon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  variant = "default",
  icon,
  children,
  className = "",
  ...props
}) => {
  const variantClass = variant === "primary" ? "btn-primary" : "";
  return (
    <button className={`btn ${variantClass} ${className}`} {...props}>
      {icon && <span className="btn-icon">{icon}</span>}
      {children && <span>{children}</span>}
    </button>
  );
};
