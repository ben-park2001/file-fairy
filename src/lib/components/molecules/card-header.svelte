<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { X } from "lucide-svelte";
  import Icon from "$lib/components/atoms/icon.svelte";
  import Text from "$lib/components/atoms/text.svelte";
  import type { BaseProps } from "$lib/types/components";

  interface Props extends BaseProps {
    title: string;
    icon: any; // Lucide icon component
    onClose: () => void;
  }

  const { title, icon, onClose, class: className }: Props = $props();
</script>

<div class="flex items-center justify-between p-4 border-b {className}">
  <div class="flex items-center gap-2">
    <Icon {icon} size="sm" class="text-blue-500" />
    <Text text={title} weight="medium" />
  </div>
  <Button variant="ghost" size="sm" onclick={onClose} class="h-8 w-8 p-0">
    <Icon icon={X} size="sm" />
  </Button>
</div>
