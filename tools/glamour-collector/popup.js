// The list: each entry with a gender choice (ANY, male or female), the classes it's for, its lines to edit, and a
// remove button; the whole list exported as glamours.txt for Kawari (docs/src/fork/party-bots.md, "Glamours").

const list = document.getElementById("entries");
const statusLine = document.getElementById("status");

function say(text) {
  statusLine.textContent = text;
  clearTimeout(say.timer);
  say.timer = setTimeout(() => (statusLine.textContent = ""), 4000);
}

async function load() {
  const { entries = [] } = await chrome.storage.local.get("entries");
  return entries;
}

async function save(entries) {
  await chrome.storage.local.set({ entries });
}

function showCounts(entries) {
  const count = (gender) => entries.filter((entry) => entry.gender === gender).length;
  document.getElementById("counts").textContent =
    `${entries.length} glamours · ${count("male")} male · ${count("female")} female · ${count("any")} ANY`;
  document.getElementById("empty").style.display = entries.length ? "none" : "";
}

// The entry's pieces' lines, without its header or Class line (the title, gender and classes have their own controls.)
function bodyText(entry) {
  return kawariEntryText({ ...entry, classes: [] }).split("\n").slice(1).join("\n");
}

function render(entries) {
  list.textContent = "";
  showCounts(entries);
  for (const entry of entries) {
    const item = document.createElement("li");

    const row = document.createElement("div");
    row.className = "row";
    const link = document.createElement("a");
    link.href = entry.url;
    link.target = "_blank";
    link.textContent = entry.title;
    link.title = entry.url;

    const gender = document.createElement("select");
    for (const [value, label] of [
      ["any", "ANY"],
      ["male", "Male"],
      ["female", "Female"],
    ]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      gender.appendChild(option);
    }
    gender.value = entry.gender;
    gender.className = entry.gender === "any" ? "any" : "";
    gender.addEventListener("change", async () => {
      const all = await load();
      const stored = all.find((other) => other.id === entry.id);
      if (stored) stored.gender = gender.value;
      await save(all);
      gender.className = gender.value === "any" ? "any" : "";
      showCounts(all);
    });

    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "quiet";
    remove.textContent = "Remove";
    remove.addEventListener("click", async () => {
      await save((await load()).filter((other) => other.id !== entry.id));
      render(await load());
    });

    row.append(link, gender, remove);

    // The classes or jobs it's for (names or abbreviations, as the game writes them); empty: any class.
    const classes = document.createElement("input");
    classes.type = "text";
    classes.className = "classes";
    classes.placeholder = "Class: any (or e.g. BLM, Scholar)";
    classes.title = "Classes or jobs this glamour is for, separated by commas; empty for any class";
    classes.value = (entry.classes || []).join(", ");
    classes.addEventListener("change", async () => {
      const all = await load();
      const stored = all.find((other) => other.id === entry.id);
      if (!stored) return;
      stored.classes = kawariNames(classes.value);
      classes.value = stored.classes.join(", ");
      await save(all);
      say(stored.classes.length ? `${stored.title}: ${stored.classes.join(", ")} only.` : `${stored.title}: any class.`);
    });

    const lines = document.createElement("textarea");
    lines.value = bodyText(entry);
    lines.spellcheck = false;
    lines.title = "One piece per line: Slot: <item> | <dye> | <second dye>";
    const warn = document.createElement("div");
    warn.className = "warn";
    if (entry.pieces.length === 0) warn.textContent = "No gear was read from the page: type the pieces in.";
    lines.addEventListener("change", async () => {
      const all = await load();
      const index = all.findIndex((other) => other.id === entry.id);
      if (index < 0) return;
      const header = kawariEntryText({ ...all[index], pieces: [] });
      const edited = kawariParseEntryText(`${header}\n${lines.value}`, all[index]);
      all[index] = edited;
      await save(all);
      classes.value = (edited.classes || []).join(", ");
      lines.value = bodyText(edited);
      warn.textContent = edited.pieces.length === 0 ? "No pieces: this glamour will be left out." : "";
      say(`Saved ${edited.title}.`);
    });

    item.append(row, classes, lines, warn);
    list.appendChild(item);
  }
}

document.getElementById("export").addEventListener("click", async () => {
  const entries = await load();
  const blob = new Blob([kawariListText(entries)], { type: "text/plain" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "glamours.txt";
  link.click();
  setTimeout(() => URL.revokeObjectURL(link.href), 10000);
  say(`Exported ${entries.length} glamours.`);
});

document.getElementById("copy").addEventListener("click", async () => {
  await navigator.clipboard.writeText(kawariListText(await load()));
  say("List copied.");
});

document.getElementById("clear").addEventListener("click", async (event) => {
  const button = event.currentTarget;
  if (button.dataset.armed !== "1") {
    button.dataset.armed = "1";
    button.textContent = "Really clear?";
    setTimeout(() => {
      button.dataset.armed = "";
      button.textContent = "Clear…";
    }, 3000);
    return;
  }
  await save([]);
  render([]);
  say("List cleared.");
});

load().then(render);
