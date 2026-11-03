/** Racehorse naming rule (mirrors the Rust `is_valid_name`): 2-9 full-width katakana, "ー" not first. */
export const NAME_MIN_CHARS = 2;
export const NAME_MAX_CHARS = 9;
export const NAME_RULE = `全角カタカナ${NAME_MIN_CHARS}〜${NAME_MAX_CHARS}文字`;

export function isValidHorseName(name: string): boolean {
  const chars = Array.from(name);
  return (
    chars.length >= NAME_MIN_CHARS &&
    chars.length <= NAME_MAX_CHARS &&
    chars[0] !== "ー" &&
    chars.every((c) => (c >= "ァ" && c <= "ヶ") || c === "ー")
  );
}
