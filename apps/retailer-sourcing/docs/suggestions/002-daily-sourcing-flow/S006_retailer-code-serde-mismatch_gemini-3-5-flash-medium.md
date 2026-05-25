# S006 — `RetailerCode` Serde representation mismatch with `as_string` and `From<String>`

| Field                    | Value                                                                                                                                                                                                                                                                                             |
|--------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                                                     |
| Decision                 | accepted                                                                                                                                                                                                                                                                                          |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (File-by-file entry for `retailer-guild` now uses `#[serde(into = "String", try_from = "String")]`; specifies replacing panicking `From<String>` with `TryFrom<String>` and adding `From<RetailerCode> for String`) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                                        |
| Author                   | Gemini, gemini-3.5-flash, medium                                                                                                                                                                                                                                                                  |
| Reviewer                 |                                                                                                                                                                                                                                                                                                   |

## Issue

The implementation plan specifies:
> `lib/retailer-guild/src/lib.rs` — add `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]` to `RetailerCode`. Add `serde` to its dependencies.

By default, deriving `Serialize` and `Deserialize` on an enum with uppercase snake_case variants such as `MINISFORUM_EU` results in a JSON string representation of `"MINISFORUM_EU"`.

However, the existing `RetailerCode` implementation in `lib/retailer-guild/src/lib.rs` defines a custom mapping:
- `as_string()` returns `"MinisForumEU"`, `"MinisForumUS"`, etc.
- `From<String>` parses `"MinisForumEU"` and panics on `"MINISFORUM_EU"`.

If `RetailerCode` is serialized using a default derived implementation (e.g. within command or event payloads), but other parts of the system or database interact with `RetailerCode` via its string representation, this mismatch will cause parsing failures and runtime panics.

## Suggestion

Ensure `RetailerCode` is serialized and deserialized using its established PascalCase string representation. Instead of a plain derive, implement `Serialize` and `Deserialize` using `serde(into = "String", try_from = "String")` or write custom Serde implementations.

For example, implement `TryFrom<String> for RetailerCode` (replacing the panicking `From<String>` with a safe conversion) or `From<RetailerCode> for String`:

```rust
impl From<RetailerCode> for String {
    fn from(code: RetailerCode) -> Self {
        code.as_string().to_string()
    }
}

// Convert From<String> to TryFrom<String> to support safe deserialization:
impl TryFrom<String> for RetailerCode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "MinisForumEU" => Ok(RetailerCode::MINISFORUM_EU),
            "MinisForumUS" => Ok(RetailerCode::MINISFORUM_US),
            "MinisForumUK" => Ok(RetailerCode::MINISFORUM_UK),
            "MinisForumFR" => Ok(RetailerCode::MINISFORUM_FR),
            "MinisForumCA" => Ok(RetailerCode::MINISFORUM_CA),
            "MinisForumAU" => Ok(RetailerCode::MINISFORUM_AU),
            other => Err(format!("unknown RetailerCode: {other}")),
        }
    }
}
```

Then configure the enum to use them:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum RetailerCode {
    ...
}
```
