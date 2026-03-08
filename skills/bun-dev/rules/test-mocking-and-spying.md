# test-mocking-and-spying

## Why

Bun’s test runner includes first-class mocking/spying. Use it to avoid adding extra test frameworks unless you need ecosystem-specific features.

Newer Bun versions (including v1.3.10) also support `Symbol.dispose` for mocks/spies, which pairs well with `using` for automatic cleanup.

## Do

- Use `mock()` for function mocks.
- Use `spyOn()` for method interception.

## Don't

- Don’t build custom mock helpers until you hit a real limitation.

## Examples

```ts
import { expect, mock, spyOn, test } from "bun:test";

test("mock + spy", () => {
  const fn = mock((x: number) => x + 1);
  expect(fn(1)).toBe(2);
  expect(fn).toHaveBeenCalledWith(1);

  const obj = { method: () => "real" };
  const spy = spyOn(obj, "method").mockReturnValue("fake");
  expect(obj.method()).toBe("fake");
  expect(spy).toHaveBeenCalled();
});
```
