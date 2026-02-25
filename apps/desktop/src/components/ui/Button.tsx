import type { ButtonHTMLAttributes } from "react";

import { cn } from "./cn";

type ButtonVariant = "default" | "primary";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: ButtonVariant;
    unstyled?: boolean;
};

export function Button({
    className,
    variant = "default",
    unstyled = false,
    type = "button",
    ...props
}: ButtonProps) {
    const resolvedClassName = unstyled
        ? className
        : cn("ui-button", variant === "primary" && "is-primary", className);

    return <button type={type} className={resolvedClassName} {...props} />;
}
