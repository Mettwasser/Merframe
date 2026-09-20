import { describe, expect, it } from "vitest";
import { visibleResources } from "@/lib/resource-filters";
import type { ResourceRow } from "@/types";

function resource(name: string, owned: number, required: number): ResourceRow {
  return {
    unique_name: `/Lotus/Types/Items/MiscItems/${name.replaceAll(" ", "")}`,
    name,
    image_name: null,
    owned,
    required,
    deficit: Math.max(required - owned, 0),
    used_by: [],
  };
}

const resources = [
  resource("Orokin Cell", 40, 30),
  resource("Hexenon", 5, 25),
  resource("Alloy Plate", 900, 600),
  resource("Neurodes", 1, 12),
];

function names(rows: ResourceRow[]): string[] {
  return rows.map((row) => row.name);
}

describe("visibleResources", () => {
  it("puts the largest shortfall first, then sorts by name", () => {
    expect(names(visibleResources(resources, "all", ""))).toEqual([
      "Hexenon",
      "Neurodes",
      "Alloy Plate",
      "Orokin Cell",
    ]);
  });

  it("keeps only resources that are short", () => {
    expect(names(visibleResources(resources, "short", ""))).toEqual([
      "Hexenon",
      "Neurodes",
    ]);
  });

  it("matches the search anywhere in the name, ignoring case and padding", () => {
    expect(names(visibleResources(resources, "all", "  NEUR "))).toEqual([
      "Neurodes",
    ]);
    expect(visibleResources(resources, "short", "orokin")).toEqual([]);
  });

  it("leaves the payload order untouched", () => {
    visibleResources(resources, "all", "");
    expect(names(resources)[0]).toBe("Orokin Cell");
  });
});
