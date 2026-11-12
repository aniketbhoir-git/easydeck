# EasyDeck for LLMs (stub)

> Status: stub. Issue R6-3 fills this page. Until then, treat it as a promise, not
> a reference.

## Purpose

A user points any LLM at this page and says what deck they want. The LLM builds or edits the
deck by changing the deck file only. It never touches app code. The user approves each change.

## What this page will contain

1. **The contract.** The deck file is JSON that follows `schema/deck.schema.json`. A change
   that fails validation applies nothing.
2. **The tools.** The local MCP server (on `127.0.0.1` only) offers `deck_get`,
   `deck_schema`, `deck_validate`, `deck_propose` and, when the user turns it on,
   `system_inventory` (installed games and programs). This page will give the exact input
   and output shape of each.
3. **The workflow.** Read the schema, read the current deck, send a full new deck to
   `deck_propose`, wait for the user. The proposal applies only after a tap in the app.
4. **Widget and action recipes.** Short, tested examples: a Steam launch tile, an `exe` tile
   with fallback, a key macro, a stat gauge, a media widget, a web embed, a page that opens
   when a game starts.
5. **Limits and safety.** An LLM cannot add an action or a network address without the
   user seeing it. Shell
   actions use an allow-list only. No secrets in a deck. The LLM must not ask for any.
6. **Common mistakes.** Overlapping grid positions, duplicate ids, unknown widget types, and how
   the validator reports each.

## Until then

Read `schema/deck.schema.json` and `docs/SPEC.md` §4. Both describe the deck shape.
