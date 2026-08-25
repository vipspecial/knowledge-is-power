export interface AiChangeGroup {
  id: number;
  original: string[];
  revised: string[];
  changed: boolean;
}

const MAX_LCS_CELLS = 120_000;

function lines(value: string): string[] {
  return value.trim().split("\n");
}

function fallbackGroups(original: string[], revised: string[]): AiChangeGroup[] {
  let prefix = 0;
  while (prefix < original.length && prefix < revised.length && original[prefix] === revised[prefix]) {
    prefix += 1;
  }

  let suffix = 0;
  while (
    suffix < original.length - prefix
    && suffix < revised.length - prefix
    && original[original.length - suffix - 1] === revised[revised.length - suffix - 1]
  ) {
    suffix += 1;
  }

  return [
    { id: 0, original: original.slice(0, prefix), revised: revised.slice(0, prefix), changed: false },
    {
      id: 1,
      original: original.slice(prefix, original.length - suffix),
      revised: revised.slice(prefix, revised.length - suffix),
      changed: true,
    },
    {
      id: 2,
      original: suffix ? original.slice(original.length - suffix) : [],
      revised: suffix ? revised.slice(revised.length - suffix) : [],
      changed: false,
    },
  ].filter((group) => group.original.length || group.revised.length);
}

/** 以行为单位生成变更组，避免仅为预览引入体积较大的 diff 依赖。 */
export function createAiChangeGroups(originalText: string, revisedText: string): AiChangeGroup[] {
  const original = lines(originalText);
  const revised = lines(revisedText);
  if (original.length * revised.length > MAX_LCS_CELLS) return fallbackGroups(original, revised);

  const columns = revised.length + 1;
  const table = new Uint16Array((original.length + 1) * columns);
  for (let left = original.length - 1; left >= 0; left -= 1) {
    for (let right = revised.length - 1; right >= 0; right -= 1) {
      const index = left * columns + right;
      table[index] = original[left] === revised[right]
        ? table[(left + 1) * columns + right + 1] + 1
        : Math.max(table[(left + 1) * columns + right], table[left * columns + right + 1]);
    }
  }

  const operations: Array<{ kind: "same" | "remove" | "add"; value: string }> = [];
  let left = 0;
  let right = 0;
  while (left < original.length && right < revised.length) {
    if (original[left] === revised[right]) {
      operations.push({ kind: "same", value: original[left] });
      left += 1;
      right += 1;
    } else if (table[(left + 1) * columns + right] >= table[left * columns + right + 1]) {
      operations.push({ kind: "remove", value: original[left] });
      left += 1;
    } else {
      operations.push({ kind: "add", value: revised[right] });
      right += 1;
    }
  }
  while (left < original.length) operations.push({ kind: "remove", value: original[left++] });
  while (right < revised.length) operations.push({ kind: "add", value: revised[right++] });

  const groups: AiChangeGroup[] = [];
  for (const operation of operations) {
    const changed = operation.kind !== "same";
    let group = groups[groups.length - 1];
    if (!group || group.changed !== changed) {
      group = { id: groups.length, original: [], revised: [], changed };
      groups.push(group);
    }
    if (operation.kind !== "add") group.original.push(operation.value);
    if (operation.kind !== "remove") group.revised.push(operation.value);
  }
  return groups;
}

export function composeAiChanges(groups: AiChangeGroup[], accepted: ReadonlySet<number>): string {
  return groups
    .flatMap((group) => (group.changed && accepted.has(group.id) ? group.revised : group.original))
    .join("\n")
    .trim();
}
