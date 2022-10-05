export const toOptionValues = <
  T extends { value: string | number; label: string },
>(
  options: T[],
  ids: number[],
): T[] =>
  options.filter((opt) =>
    typeof opt.value === "number" ? ids.includes(opt.value) : false,
  );
