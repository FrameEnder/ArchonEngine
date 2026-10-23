// Reads the glamour on the page: its title, who it's for, and each piece's slot, item and dyes.
//
// This first version doesn't know Eorzea Collection's markup (it was written without seeing the page's HTML), so it
// reads the page by its visible text rather than class names: it looks for the slot labels (Head, Body, Hands...),
// and takes each label's item and dyes from the text beside it. If a page comes out wrong, the popup's "Copy page
// HTML" gives the gear section's HTML to send over, and this function gets matched to the real markup. Every entry
// can also be corrected by hand in the popup.

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
  "right ring": "Ring",
  "left ring": "Ring",
  weapon: "Weapon",
  "main hand": "Weapon",
  "off hand": "Off Hand",
  "off-hand": "Off Hand",
  offhand: "Off Hand",
};

function kawariCleanText(text) {
  return (text || "").replace(/\s+/g, " ").trim();
}

function kawariTitle(doc) {
  const h1 = doc.querySelector("h1");
  if (h1 && kawariCleanText(h1.textContent)) return kawariCleanText(h1.textContent);
  const og = doc.querySelector('meta[property="og:title"]');
  const title = kawariCleanText(og ? og.content : doc.title);
  return title.replace(/\s*[|\-–]\s*Eorzea Collection.*$/i, "").trim() || "Glamour";
}

// "male", "female" or "any", from a label on the page saying so.
function kawariGender(doc) {
  const words = [];
  for (const element of doc.querySelectorAll("a, span, li, div, p, small, strong")) {
    if (element.children.length > 0) continue;
    const text = kawariCleanText(element.textContent).toLowerCase();
    if (text === "male" || text === "♂" || text === "female" || text === "♀") words.push(text);
  }
  const male = words.some((word) => word === "male" || word === "♂");
  const female = words.some((word) => word === "female" || word === "♀");
  if (male && !female) return "male";
  if (female && !male) return "female";
  return "any";
}

// The pieces next to each slot label: the label's container (the nearest ancestor holding more than the label, and no
// other slot's label) holds the item's name first and then its dyes.
function kawariPieces(doc) {
  const labels = [];
  for (const element of doc.querySelectorAll("body *")) {
    if (element.children.length > 0) continue;
    const slot = KAWARI_SLOT_NAMES[kawariCleanText(element.textContent).toLowerCase().replace(/:$/, "")];
    if (slot) labels.push({ element, slot });
  }
  const labelElements = new Set(labels.map((label) => label.element));
  const pieces = [];
  const seen = new Set();
  for (const { element, slot } of labels) {
    let container = element.parentElement;
    for (let depth = 0; container && depth < 5; depth += 1, container = container.parentElement) {
      const others = [...container.querySelectorAll("*")].filter(
        (other) => labelElements.has(other) && other !== element,
      );
      if (others.length > 0) {
        container = null;
        break;
      }
      const texts = kawariTexts(container).filter((text) => text.element !== element);
      if (texts.length > 0) {
        const values = texts.map((text) => text.value);
        const key = `${slot}:${values[0]}`;
        if (!seen.has(key)) {
          seen.add(key);
          pieces.push({ slot, item: values[0], dyes: values.slice(1, 3) });
        }
        break;
      }
    }
  }
  return pieces;
}

// The visible text pieces inside an element, in order (each leaf element's text, or an image's alt text.)
function kawariTexts(root) {
  const texts = [];
  for (const element of root.querySelectorAll("*")) {
    if (element.children.length > 0 && element.tagName !== "A") continue;
    let value = kawariCleanText(element.textContent);
    if (!value && element.tagName === "IMG") value = kawariCleanText(element.alt || element.title);
    if (!value || value.length > 80) continue;
    texts.push({ element, value });
  }
  return texts;
}

function kawariParseGlamour(doc) {
  return {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    url: doc.location ? doc.location.href.split("#")[0] : "",
    title: kawariTitle(doc),
    gender: kawariGender(doc),
    classes: [],
    pieces: kawariPieces(doc),
    added: new Date().toISOString(),
  };
}

// The gear section's HTML (or the page's main content), to send over when a page reads wrong.
function kawariPageHtml(doc) {
  const labels = [...doc.querySelectorAll("body *")].filter(
    (element) =>
      element.children.length === 0 &&
      KAWARI_SLOT_NAMES[kawariCleanText(element.textContent).toLowerCase().replace(/:$/, "")],
  );
  let section = labels.length > 0 ? labels[0].parentElement : null;
  while (section && labels.some((label) => !section.contains(label))) section = section.parentElement;
  const root = section || doc.querySelector("main") || doc.body;
  return root.outerHTML;
}

if (typeof module !== "undefined") {
  module.exports = { kawariParseGlamour, kawariPageHtml };
}
