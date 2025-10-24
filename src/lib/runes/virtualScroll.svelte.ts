export function createVirtualScroll<T>(options: {
  items: () => T[];
  itemHeight?: number;
  containerHeight?: number;
  overscan?: number;
}) {
  const itemHeight = options.itemHeight ?? 34;
  const containerHeight = options.containerHeight ?? 624;
  const overscan = options.overscan ?? 5;

  let scrollTop = $state(0);

  const visibleStart = $derived(
    Math.max(0, Math.floor(scrollTop / itemHeight) - overscan),
  );

  const visibleEnd = $derived(
    Math.min(
      options.items().length,
      Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan,
    ),
  );

  const visibleItems = $derived(
    options.items().slice(visibleStart, visibleEnd),
  );

  const totalHeight = $derived(options.items().length * itemHeight);
  const offsetY = $derived(visibleStart * itemHeight);

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  return {
    get scrollTop() {
      return scrollTop;
    },
    set scrollTop(v: number) {
      scrollTop = v;
    },
    get visibleStart() {
      return visibleStart;
    },
    get visibleEnd() {
      return visibleEnd;
    },
    get visibleItems() {
      return visibleItems;
    },
    get totalHeight() {
      return totalHeight;
    },
    get offsetY() {
      return offsetY;
    },
    handleScroll,
  };
}
