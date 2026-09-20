import type { ResourceRow } from "@/types";

export type ShowFilter = "all" | "short";

export function visibleResources(
  resources: ResourceRow[],
  show: ShowFilter,
  search: string,
): ResourceRow[] {
  const needle = search.trim().toLowerCase();
  return resources
    .filter(
      (row) =>
        (show === "all" || row.deficit > 0) &&
        row.name.toLowerCase().includes(needle),
    )
    .sort((left, right) =>
      left.deficit === right.deficit
        ? left.name.localeCompare(right.name)
        : right.deficit - left.deficit,
    );
}
