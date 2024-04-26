# Random Tables Parser

This is a work in progress parser/generator of random tables using Rust and `nom` (a parser combinator).

Table definitions look like the following (entries can refer to other tables using `{{table_name}}`):

```yml
---
title: Colors
id: color
---
1: Red
1: Blue
1: Green

---
title: Shapes
id: shape
---
1: Circle
1: Square
1: Triangle

---
title: Colored Shapes
id: colored_shape
---
1: {{color}} {{shape}}
```

Entry frequency can be adjusted by specifying a different weight for some rows:

```yml
---
title: Variable Frequency
id: variable_frequency
---
60: common
30: uncommon
15: rare
5: epic
3: legendary
1: mythic
```

## CLI
| Flag | Description |
|-|-|
| `-d`, `--definition` | the file name of the `.tbl` file in the `src/tables` folder to use |
| `-t`, `--table` | the name of the table within the definition to generate variants for |
| `-c`, `--count` | (default 10): how many to generate |
| `--debug` | turn on debug logging |

## TODO
- [ ] optional interpolations?
- [ ] built-ins
- [ ] extensibility
- [x] better handling of articles (a/an)
- [x] filters (e.g. capitalize, for articles)
- [x] consider switching to weights instead of numeric indices - easier to adjust frequency
- [ ] some kind of UI?
- [ ] wasm build
- [x] CLI with clap
  - [x] load .tbl from file
  - [x] execute some gen/gen_many command
  - [ ] interactive? (browse tables, issue multiple gen commands, history)
- [ ] versioning (version in frontmatter)
  - [ ] versioned parsers
- [ ] fix a table value

### Future
- parameters/filtering (e.g. count, distinct, conjunctions, context-aware of previous matches)
- better validation
- distribution? referencing tables other people maintain
  - forking is useful for tweaking, but a ref could mean getting updates without "pull"ing
  - compatibility/versioning (hash of contents?)

## Example Output

### Potions

- A vibrant green liquid contained within a tall decanter. It smells like coal and it sparkles with blue flecks.
- A fresh white goo floating in a tall jug. It smells like a fresh apple pie and it roils and bubbles energetically.
- A questionable yellow-black-white liquid resting in a thin-rimmed jug. It smells like regret and it roils and bubbles energetically.
- A brand new multi-colored-green liquid held by a double-spouted vial. It smells like regret and it's a little bit sticky.
- A questionable green goo resting in a tear-shaped flask. It smells like a foot and it seems to be calling you to drink it.
- A stale white-pink liquid sloshing around in a tear-shaped beaker. It smells like boiled sweets and it shimmers and swirls curiously.
- A fresh red gelatin delicately stirring within a tear-shaped bottle. It smells like sulphur and resembles a failed science experiment.
- A musty multi-colored sludge swirling within a long and narrow decanter. It smells like death and it's warm to the touch.
- A whimsical chromatic goo trapped in a round flask. It smells like a summer breeze and reminds you of someone you used to know.
- A musty purple sludge bubbling within a rectangular vial. It smells like a foot and it pops and crackles.

### NPCs

- Bilbo Elessar is a commoner from a small merchant village outside Longsaddle, known as a(n) master barkeep.
- Pippin Brandybuck is a Chaotic Evil Dwarf Paladin originally from a large desert city, known as a(n)untrustworthy - grandmaster butcher.
- Gimli Took is a Neutral Human Rogue originally from a large island city, known as a(n) sketchy apprentice baker.
- Bilbo the Grey is a commoner from a small jungle city, known as a(n) expert fisherman.
Boromir the Grey is a Lawful Neutral Gnome Paladin originally from a small tundra city, known as a(n) famousapprentice hunter.
- Boromir Oakenshield is a commoner from a small swamp city, known as a(n) journeyman gambler.
Sam Hornblower is a Lawful Evil Elf Paladin originally from the wilds outside Waterdeep, known as a(n) well-liked novice cook.
- Legolas Greenleaf is a commoner from a small logging village outside Triboar, known as a(n) expert banker.
- Boromir Gamgee is a Neutral Good Half-Dwarf Paladin originally from a small merchant village outside Waterdeep, known as a(n) tolerated novice baker.
- Gimli Gamgee is a commoner from a small swamp city, known as a(n) novice logger.