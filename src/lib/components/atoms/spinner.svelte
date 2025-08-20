<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    show: boolean;
    size?: "sm" | "md" | "lg";
    color?: "primary" | "secondary" | "muted";
    class?: string;
  }

  const {
    show,
    size = "md",
    color = "primary",
    class: className,
  }: Props = $props();

  const sizeClasses = {
    sm: "h-4 w-4",
    md: "h-6 w-6",
    lg: "h-8 w-8",
  };

  const colorClasses = {
    primary: "text-primary",
    secondary: "text-secondary",
    muted: "text-muted-foreground",
  };
</script>

{#if show}
  <div
    class={cn(
      "animate-spin rounded-full border-2 border-current border-t-transparent",
      sizeClasses[size],
      colorClasses[color],
      className
    )}
  ></div>
{/if}
