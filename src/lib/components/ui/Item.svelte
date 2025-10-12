<script lang="ts">
  let { width, children, ...restProps } = $props();
</script>

<div class="flex shrink-0 items-center text-sm" style="width: {width}px">
  {@render children?.()}
</div>
