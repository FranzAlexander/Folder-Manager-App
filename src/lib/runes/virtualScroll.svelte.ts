// export class VirtualScroll<T> {
//   scrollTop = $state(0);
//   itemHeight = $state(34);
//   containerHeight = $state(624);

//   private overscan: number;
//   private items!: () => T[];

//   constructor(options: {
//     items: () => T[];
//     itemHeight?: number;
//     containerHeight?: number;
//     overscan?: number;
//   }) {
//     this.items = options.items;
//     if (options.itemHeight) this.itemHeight = options.itemHeight;
//     if (options.containerHeight) this.containerHeight = options.containerHeight;
//     this.overscan = options.overscan ?? 5;
//   }

//   visibleStart = $derived(
//     Math.max(0, Math.floor(this.scrollTop / this.itemHeight) - this.overscan),
//   );

//   visibleEnd = $derived(
//     Math.min(
//       this.items().length,
//       Math.ceil((this.scrollTop + this.containerHeight) / this.itemHeight) +
//         this.overscan,
//     ),
//   );

//   visibleItems = $derived(
//     this.items().slice(this.visibleStart, this.visibleEnd),
//   );

//   totalHeight = $derived(this.items().length * this.itemHeight);

//   offsetY = $derived(this.visibleStart * this.itemHeight);

//   // Use arrow function to preserve `this` context
//   handleScroll = (e: Event) => {
//     const target = e.target as HTMLElement;
//     this.scrollTop = target.scrollTop;
//   };
// }

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
