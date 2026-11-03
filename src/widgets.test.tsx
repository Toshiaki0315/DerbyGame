import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FormBadge, GateBadge, SideOptions } from "./widgets";

describe("the small pieces the screens share", () => {
  it("shows a horse's form in words, for the eye and for a screen reader", () => {
    render(<FormBadge horse={{ name: "Fast", condition: "great" }} />);
    expect(screen.getByLabelText("Fastの調子: 絶好調").textContent).toBe("絶好調");
  });

  it("gives each runner the gate of its place in the field", () => {
    render(
      <>
        <GateBadge index={0} count={18} name="First" />
        <GateBadge index={17} count={18} name="Last" />
      </>
    );
    expect(screen.getByLabelText(/First/).textContent).toBe("1");
    expect(screen.getByLabelText(/Last/).textContent).toBe("8");
  });

  it("groups people by side, east first, whatever order they come in", () => {
    render(
      <select aria-label="誰">
        <SideOptions
          people={[{ id: 1, region: "栗東" }, { id: 2, region: "美浦" }]}
          option={(p) => <option key={p.id}>{p.id}</option>}
        />
      </select>
    );
    const groups = Array.from(screen.getByLabelText("誰").querySelectorAll("optgroup"));
    expect(groups.map((g) => [g.label, g.textContent])).toEqual([["美浦", "2"], ["栗東", "1"]]);
  });

  it("leaves people with no side ungrouped", () => {
    const people: { id: number; region?: string }[] = [{ id: 1 }];
    render(
      <select aria-label="誰">
        <SideOptions people={people} option={(p) => <option key={p.id}>{p.id}</option>} />
      </select>
    );
    expect(screen.getByLabelText("誰").querySelectorAll("optgroup").length).toBe(0);
    expect(screen.getByRole("option").textContent).toBe("1");
  });
});
