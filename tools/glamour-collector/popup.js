// The list: each entry with a gender choice (ANY, male or female), the classes it's for, its lines to edit, and a
// remove button; the whole list sent to your Kawari Web (which writes the server's glamours.txt) or exported as the
// file (docs/src/fork/party-bots.md, "Glamours"; docs/src/fork/tools.md, "Glamour Collector").

const list = document.getElementById("entries");
const statusLine = document.getElementById("status");

// The server's class list (`classList` in storage), for the dropdown; none until connected.
let classList = null;

function say(text, isError = false) {
  statusLine.textContent = text;
  statusLine.className = isError ? "error" : "";
  clearTimeout(say.timer);
  say.timer = setTimeout(() => (statusLine.textContent = ""), isError ? 9000 : 4000);
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

// The Class chips and dropdown in `container` for `names`; `change(names)` saves a new list.
function showClasses(container, names, change) {
  container.textContent = "";
  for (const name of names) {
    const chip = document.createElement("span");
    const known = kawariKnownClass(classList, name);
    chip.className = known ? "chip" : "chip unknown";
    if (!known) chip.title = "Not one of your server's combat classes or roles: bots won't wear this glamour";
    chip.textContent = name;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "×";
    remove.title = `Remove ${name}`;
    remove.addEventListener("click", () => change(names.filter((other) => other !== name)));
    chip.appendChild(remove);
    container.appendChild(chip);
  }
  const select = document.createElement("select");
  const first = document.createElement("option");
  first.value = "";
  first.textContent = names.length ? "+ Add class…" : "Any class (+ add one…)";
  select.appendChild(first);
  for (const [group, members] of kawariClassGroups(classList)) {
    const optgroup = document.createElement("optgroup");
    optgroup.label = group;
    for (const member of members) {
      const option = document.createElement("option");
      option.value = member;
      option.textContent = member;
      option.disabled = names.includes(member);
      optgroup.appendChild(option);
    }
    select.appendChild(optgroup);
  }
  if (!classList) {
    const hint = document.createElement("option");
    hint.disabled = true;
    hint.textContent = "(connect Kawari Web for the jobs)";
    select.appendChild(hint);
  }
  select.addEventListener("change", () => {
    if (select.value) change([...names, select.value]);
  });
  container.appendChild(select);
}

function render(entries) {
  list.textContent = "";
  showCounts(entries);
  for (const entry of entries) {
    const item = document.createElement("li");

    const row = document.createElement("div");
    row.className = "row";
    const link = document.createElement(entry.url ? "a" : "span");
    if (entry.url) {
      link.href = entry.url;
      link.target = "_blank";
      link.title = entry.url;
    }
    link.className = "title";
    link.textContent = entry.title;

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

    // The classes or jobs it's for, as chips (× removes one) and a dropdown to add one; none: any class.
    const classes = document.createElement("div");
    classes.className = "classes";
    const setClasses = async (names) => {
      const all = await load();
      const stored = all.find((other) => other.id === entry.id);
      if (!stored) return;
      stored.classes = names;
      await save(all);
      showClasses(classes, names, setClasses);
      say(names.length ? `${stored.title}: ${names.join(", ")} only.` : `${stored.title}: any class.`);
    };
    showClasses(classes, entry.classes || [], setClasses);

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
      showClasses(classes, edited.classes || [], setClasses);
      lines.value = bodyText(edited);
      warn.textContent = edited.pieces.length === 0 ? "No pieces: this glamour will be left out." : "";
      say(`Saved ${edited.title}.`);
    });

    item.append(row, classes, lines, warn);
    list.appendChild(item);
  }
}

// ── Kawari Web ────────────────────────────────────────────────────────────

const serverUrl = document.getElementById("server-url");
const serverToken = document.getElementById("server-token");
const serverState = document.getElementById("server-state");

function showServer(settings, state) {
  serverUrl.value = settings.url || "";
  serverToken.value = settings.token || "";
  serverState.textContent = state;
  document.getElementById("server").open = !settings.url;
}

document.getElementById("connect").addEventListener("click", async () => {
  const origin = kawariWebOrigin(serverUrl.value);
  const token = serverToken.value.trim();
  if (!origin || !token) {
    say("Type your Kawari Web's address and an administrator's token.", true);
    return;
  }
  // Chrome only asks from a click, so this comes first.
  if (!(await kawariWebAllow(origin))) {
    say(`Chrome wasn't allowed to reach ${origin}.`, true);
    return;
  }
  await chrome.storage.local.set({ kawariWeb: { url: origin, token } });
  try {
    const ping = await kawariWebCall("/api/glamours/ping");
    let state = `· connected (${ping.list.glamours} on the server)`;
    if (ping.classes) {
      classList = await kawariWebRefreshClasses();
    } else {
      state += " · no class list yet";
      say("Connected. Read the class list in Kawari Web (Dashboard → Bot glamours) to fill the dropdown.");
    }
    showServer({ url: origin, token }, state);
    if (ping.classes) say(`Connected to ${origin}; ${classList.classes.length} classes and jobs.`);
    document.getElementById("server").open = false;
    render(await load());
  } catch (error) {
    showServer({ url: origin, token }, "· not connected");
    say(error.message, true);
  }
});

document.getElementById("forget").addEventListener("click", async () => {
  const { url } = await kawariWebSettings();
  await chrome.storage.local.remove(["kawariWeb", "classList"]);
  if (url) await chrome.permissions.remove({ origins: [`${url}/*`] }).catch(() => {});
  classList = null;
  showServer({ url: "", token: "" }, "");
  render(await load());
  say("Kawari Web forgotten.");
});

document.getElementById("send").addEventListener("click", async (event) => {
  // Taken before any await: the event's currentTarget is gone after one.
  const button = event.currentTarget;
  const entries = (await load()).filter((entry) => entry.pieces.length > 0);
  if (entries.length === 0) {
    say("Nothing to send: the list is empty.", true);
    return;
  }
  if (button.dataset.armed !== "1") {
    button.dataset.armed = "1";
    button.textContent = `Replace the server's list with ${entries.length}?`;
    setTimeout(() => {
      button.dataset.armed = "";
      button.textContent = "Send to Kawari Web";
    }, 4000);
    return;
  }
  button.dataset.armed = "";
  button.textContent = "Send to Kawari Web";
  try {
    const body = await kawariWebCall("/api/glamours", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ text: kawariListText(entries) }),
    });
    say(
      `Sent: the server has ${body.list.glamours} glamours (${body.list.male} male, ${body.list.female} female, ` +
        `${body.list.any} either). Bots joining from now wear them.`,
    );
    serverState.textContent = `· connected (${body.list.glamours} on the server)`;
  } catch (error) {
    say(error.message, true);
  }
});

document.getElementById("fetch").addEventListener("click", async () => {
  try {
    const body = await kawariWebCall("/api/glamours");
    const theirs = kawariParseListText(body.text || "");
    const entries = await load();
    const key = (entry) => `${entry.title.toLowerCase()}|${entry.gender}`;
    const ours = new Set(entries.map(key));
    const added = theirs.filter((entry) => !ours.has(key(entry)));
    await save([...entries, ...added]);
    render(await load());
    say(`Loaded ${theirs.length} from the server; ${added.length} weren't in this list.`);
  } catch (error) {
    say(error.message, true);
  }
});

// ── The file ──────────────────────────────────────────────────────────────

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
  say("List cleared (the server's list is untouched).");
});

(async () => {
  const settings = await kawariWebSettings();
  ({ classList = null } = await chrome.storage.local.get("classList"));
  showServer(settings, settings.url ? (classList ? "· connected" : "· no class list yet") : "");
  render(await load());
  // Refresh the class list quietly (a game update can add jobs.)
  if (settings.url && settings.token) {
    kawariWebRefreshClasses()
      .then(async (fresh) => {
        // Only redrawn when it changed, so a line being edited isn't lost.
        if (JSON.stringify(fresh) === JSON.stringify(classList)) return;
        classList = fresh;
        render(await load());
      })
      .catch(() => {});
  }
})();
