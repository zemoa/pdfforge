export function defaultOutputName(sourceName: string) {
  return `${sourceName.replace(/\.pdf$/i, "")}-masked.pdf`;
}

export function sourceDirectory(sourcePath: string) {
  const separator = Math.max(sourcePath.lastIndexOf("/"), sourcePath.lastIndexOf("\\"));
  return separator < 0 ? "" : sourcePath.slice(0, separator || 1);
}
