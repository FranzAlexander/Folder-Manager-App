export function formatDate(date: string): string {
  return new Intl.DateTimeFormat("en-AU", {
    dateStyle: "short",
    timeStyle: "short",
  }).format(new Date(date));
}

export function formatFileSize(bytes: number | null): string | undefined {
  if (bytes === 0) return "0 bytes";
  if (bytes === null) return;
  if (bytes < 1024) return `${bytes} bytes`;

  const units = ["KB", "MB", "GB", "TB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k)) - 1;

  const size = bytes / Math.pow(k, i + 1);

  return `${size.toFixed(1)} ${units[i]}`;
}
