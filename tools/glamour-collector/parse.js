// Reads the glamour on an Eorzea Collection page: its title, who it fits, the classes it's for, and each piece's slot,
// item and dyes. Matched to the site's markup as of 2026-10-23 (read from the Vampire Reverie page and twenty others):
//
// * the title: `h1.title` (the page's other h1s are the menu's);
// * rows of `.columns`, each a `span.subtitle` label ("Model", "Intended for", "Level to Equip", "Fits") and `.tag`
//   chips; "Fits" reads "Any Gender", "Female" or "Male"; "Intended for" has an `img` whose alt is the job's
//   abbreviation and title its name ("RDM" / "Red Mage"), or a group ("Tanks", "Caster DPS", "Gatherers"), or "ALL";
// * a `.list.box` per slot: `.gear-icon-box-slot-name` (HEAD, BODY, HANDS, LEGS, FEET, EARRINGS, NECKLACE,
//   BRACELETS, RING, FACE, WEAPON, OFFHAND), and a `.list-item` per piece with the name in `.list-item-title` and
//   chips in `.list-item-description .tag`: a dye has a `has-text-dye-*` dot, "◯ Undyed" is no dye, and the rest are
//   labels ("Optional", "Seasonal Item", "Online Store Exclusive").
//
// If the site changes, the panel's "Copy page HTML" copies the gear section to send over.

const KAWARI_SLOT_NAMES = {
  head: "Head",
  body: "Body",
  hands: "Hands",
  legs: "Legs",
  feet: "Feet",
  earrings: "Ears",
  ears: "Ears",
  necklace: "Neck",
  neck: "Neck",
  bracelets: "Wrists",
  bracelet: "Wrists",
  wrists: "Wrists",
  ring: "Ring",
  rings: "Ring",
  face: "Face",
  weapon: "Weapon",
  offhand: "Off Hand",
  "off hand": "Off Hand",
};

function kawariCleanText(text) {
  return (text || "").replace(/\s+/g, " ").trim();
}

function kawariTitle(doc) {
  const title = doc.querySelector("h1.title");
  if (title && kawariCleanText(title.textContent)) return kawariCleanText(title.textContent);
  const og = doc.querySelector('meta[property="og:title"]');
  const text = kawariCleanText(og ? og.content : doc.title);
  return text.replace(/\s*\|\s*Eorzea Collection.*$/i, "").trim() || "Glamour";
}

// The info row labelled `label` ("Fits", "Intended for"), outside the site's menus.
function kawariInfoRow(doc, label) {
  for (const subtitle of doc.querySelectorAll("span.subtitle")) {
    if (subtitle.closest("nav, [class*=mega]")) continue;
    if (kawariCleanText(subtitle.textContent).toLowerCase() !== label.toLowerCase()) continue;
    const row = subtitle.closest(".columns");
    if (row) return row;
  }
  return null;
}

// "male", "female" or "any", from the "Fits" row.
function kawariGender(doc) {
  const row = kawariInfoRow(doc, "Fits");
  if (!row) return "any";
  const text = [...row.querySelectorAll(".tag, img")]
    .map((element) => (element.tagName === "IMG" ? element.title || element.alt : element.textContent))
    .join(" ")
    .toLowerCase();
  if (/\bfemale\b/.test(text)) return "female";
  if (/\bmale\b/.test(text)) return "male";
  return "any";
}

// The classes from the "Intended for" row: each icon's name ("Red Mage", "Tanks"), none for all classes.
function kawariClasses(doc) {
  const row = kawariInfoRow(doc, "Intended for");
  if (!row) return [];
  const classes = [];
  const images = [...row.querySelectorAll("img")];
  const names = images.length
    ? images.map((image) => ({ alt: kawariCleanText(image.alt), name: kawariCleanText(image.title || image.alt) }))
    : [...row.querySelectorAll(".tag")].map((tag) => ({ alt: "", name: kawariCleanText(tag.textContent) }));
  for (const { alt, name } of names) {
    if (!name || /^all$/i.test(alt) || /^all( classes)?$/i.test(name)) continue;
    if (!classes.includes(name)) classes.push(name);
  }
  return classes;
}

// A piece's dyes: the chips with a dye's dot, and "Undyed"; not the labels.
function kawariDyes(listItem) {
  const dyes = [];
  for (const chip of listItem.querySelectorAll(".list-item-description .tag")) {
    const text = kawariCleanText(chip.textContent.replace(/[⬤◯●○]/g, ""));
    if (chip.querySelector("[class*='has-text-dye']") || /^undyed$/i.test(text)) {
      if (text) dyes.push(/^undyed$/i.test(text) ? "undyed" : text);
    }
  }
  return dyes.slice(0, 2);
}

function kawariPieces(doc) {
  const pieces = [];
  for (const label of doc.querySelectorAll(".gear-icon-box-slot-name")) {
    const slot = KAWARI_SLOT_NAMES[kawariCleanText(label.textContent).toLowerCase()];
    const box = label.closest(".list.box");
    if (!slot || !box) continue;
    for (const listItem of box.querySelectorAll(".list-item")) {
      const title = listItem.querySelector(".list-item-title");
      const item = title ? kawariCleanText(title.textContent) : "";
      if (item) pieces.push({ slot, item, dyes: kawariDyes(listItem) });
    }
  }
  return pieces;
}

function kawariParseGlamour(doc) {
  return {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    url: doc.location ? doc.location.href.split("#")[0] : "",
    title: kawariTitle(doc),
    gender: kawariGender(doc),
    classes: kawariClasses(doc),
    pieces: kawariPieces(doc),
    added: new Date().toISOString(),
  };
}

// The gear section's HTML (the info rows and every slot's box), to send over if the site changes.
function kawariPageHtml(doc) {
  const parts = [kawariInfoRow(doc, "Fits"), kawariInfoRow(doc, "Intended for")].filter(Boolean);
  const boxes = [...doc.querySelectorAll(".gear-icon-box-slot-name")].map((label) => label.closest(".list.box"));
  parts.push(...boxes.filter(Boolean));
  if (parts.length === 0) return (doc.querySelector("main") || doc.body).outerHTML;
  return parts.map((part) => part.outerHTML).join("\n");
}

if (typeof module !== "undefined") {
  module.exports = { kawariParseGlamour, kawariPageHtml };
}
