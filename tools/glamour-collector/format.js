// The list's entries and how they're written out for Kawari's glamours.txt (docs/src/fork/party-bots.md, "Glamours").
//
// An entry: { id, url, title, gender: "any" | "male" | "female", classes: [names or abbreviations; empty: any class],
// pieces: [{ slot, item, dyes: [..] }], added }.

const KAWARI_SLOTS = [
  "Head",
  "Body",
  "Hands",
  "Legs",
  "Feet",
  "Ears",
  "Neck",
  "Wrists",
  "Ring",
  "Face",
  "Weapon",
  "Off Hand",
];

const KAWARI_HEADER = "# Lines starting with # are notes.";

function kawariGenderTag(gender) {
  if (gender === "male") return "(male)";
  if (gender === "female") return "(female)";
  return "(ANY)";
}

// One entry as glamours.txt lines.
function kawariEntryText(entry) {
  const lines = [`== ${entry.title} ${kawariGenderTag(entry.gender)}`];
  const classes = (entry.classes || []).filter((name) => name);
  if (classes.length > 0) lines.push(`Class: ${classes.map((name) => `<${name}>`).join(" | ")}`);
  const order = (piece) => {
    const index = KAWARI_SLOTS.indexOf(piece.slot);
    return index < 0 ? KAWARI_SLOTS.length : index;
  };
  const pieces = [...entry.pieces].sort((a, b) => order(a) - order(b));
  for (const piece of pieces) {
    if (!piece.item) continue;
    const parts = [`<${piece.item}>`, ...piece.dyes.filter((dye) => dye).map((dye) => `<${dye}>`)];
    lines.push(`${piece.slot}: ${parts.join(" | ")}`);
  }
  return lines.join("\n");
}

// The whole list, ready to save as glamours.txt.
function kawariListText(entries) {
  return [KAWARI_HEADER, ...entries.map(kawariEntryText)].join("\n\n") + "\n";
}

// The names in "<a> | <b>" (or "a, b"), without the brackets.
function kawariNames(text) {
  return text
    .split(/[|,]/)
    .map((part) => part.trim().replace(/^<|>$/g, "").trim())
    .filter((part) => part);
}

// Reads one entry's lines back (after editing them in the popup): the header's title and gender, its classes and its
// pieces.
function kawariParseEntryText(text, fallback) {
  const entry = { ...fallback, classes: [], pieces: [] };
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    if (line.startsWith("==")) {
      let header = line.slice(2).trim();
      const tag = header.match(/\((male|female|any|either)\)\s*$/i);
      if (tag) {
        const word = tag[1].toLowerCase();
        entry.gender = word === "male" || word === "female" ? word : "any";
        header = header.slice(0, tag.index).trim();
      }
      entry.title = header;
      continue;
    }
    const colon = line.indexOf(":");
    const slot = colon > 0 ? line.slice(0, colon).trim() : "Body";
    if (/^(class|classes|job|jobs)$/i.test(slot)) {
      entry.classes.push(...kawariNames(line.slice(colon + 1)));
      continue;
    }
    const parts = (colon > 0 ? line.slice(colon + 1) : line)
      .split("|")
      .map((part) => part.trim().replace(/^<|>$/g, "").trim())
      .filter((part) => part);
    if (parts.length === 0) continue;
    entry.pieces.push({ slot, item: parts[0], dyes: parts.slice(1) });
  }
  return entry;
}

// A whole list's text (glamours.txt, or what ArchonEngine ControlPanel sends back) as entries.
function kawariParseListText(text) {
  const blocks = [];
  for (const line of text.split("\n")) {
    if (line.trim().startsWith("==")) blocks.push([line]);
    else if (blocks.length) blocks[blocks.length - 1].push(line);
  }
  return blocks
    .map((block, index) =>
      kawariParseEntryText(block.join("\n"), {
        id: `${Date.now()}-${index}-${Math.random().toString(36).slice(2, 6)}`,
        url: "",
        title: "Glamour",
        gender: "any",
        added: new Date().toISOString(),
      }),
    )
    .filter((entry) => entry.pieces.length > 0);
}

if (typeof module !== "undefined") {
  module.exports = {
    KAWARI_SLOTS,
    kawariEntryText,
    kawariListText,
    kawariNames,
    kawariParseEntryText,
    kawariParseListText,
  };
}
