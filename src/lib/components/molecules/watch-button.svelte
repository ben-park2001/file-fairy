<script lang="ts">
  import { Eye } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import Icon from "$lib/components/atoms/icon.svelte";

  interface Props {
    onClick: () => void;
    title?: string;
    class?: string;
  }

  const {
    onClick,
    title = "Manage watched folders",
    class: className,
  }: Props = $props();
</script>

<Button
  type="button"
  variant="ghost"
  size="sm"
  onclick={onClick}
  {title}
  class="h-8 w-8 p-0 text-muted-foreground hover:text-foreground {className}"
>
  <Icon icon={Eye} size="sm" />
</Button>
