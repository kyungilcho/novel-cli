import type { ButtonHTMLAttributes, ReactNode } from "react";

import { Button } from "./Button";
import { cn } from "./cn";

type IconButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
    icon: string;
    children?: ReactNode;
    iconClassName?: string;
    dot?: boolean;
    unstyled?: boolean;
    variant?: "default" | "primary";
};

export function IconButton({
    icon,
    children,
    iconClassName,
    dot = false,
    className,
    unstyled = false,
    variant = "default",
    ...props
}: IconButtonProps) {
    return (
        <Button
            className={className}
            unstyled={unstyled}
            variant={variant}
            {...props}
        >
            <span className={cn("material-symbols-outlined", iconClassName)}>{icon}</span>
            {dot && <span className="activity-dot" />}
            {children}
        </Button>
    );
}
