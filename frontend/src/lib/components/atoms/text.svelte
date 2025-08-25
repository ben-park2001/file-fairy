<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    text: string;
    size?: "xs" | "sm" | "md" | "lg" | "xl";
    weight?: "normal" | "medium" | "semibold" | "bold";
    color?: "primary" | "secondary" | "muted" | "error" | "success";
    class?: string;
  }

  const {
    text,
    size = "md",
    weight = "normal",
    color = "primary",
    class: className,
  }: Props = $props();

  const sizeClasses = {
    xs: "text-xs",
    sm: "text-sm",
    md: "text-base",
    lg: "text-lg",
    xl: "text-xl",
  };

  const weightClasses = {
    normal: "font-normal",
    medium: "font-medium",
    semibold: "font-semibold",
    bold: "font-bold",
  };

  const colorClasses = {
    primary: "text-foreground",
    secondary: "text-secondary-foreground",
    muted: "text-muted-foreground",
    error: "text-destructive",
    success: "text-green-600",
  };
</script>

<span
  class={cn(
    sizeClasses[size],
    weightClasses[weight],
    colorClasses[color],
    className
  )}
>
  {text}
</span>
