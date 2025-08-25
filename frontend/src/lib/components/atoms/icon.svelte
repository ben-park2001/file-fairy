<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    icon: any;
    size?: "sm" | "md" | "lg";
    class?: string;
    [key: string]: any;
  }

  const {
    icon: Icon,
    size = "md",
    class: className,
    ...props
  }: Props = $props();

  const sizeClasses = {
    sm: "h-4 w-4",
    md: "h-5 w-5",
    lg: "h-6 w-6",
  };
</script>

<Icon class={cn(sizeClasses[size], className)} {...props} />
