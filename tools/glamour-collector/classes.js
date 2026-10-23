// The Class dropdown's choices. The classes and jobs come from your Kawari Web (GET /api/glamours/classes, read from
// the server's game install), kept in chrome.storage.local as `classList` once connected, so no class names are
// written into the extension. The role groups are Eorzea Collection's own words for them ("Intended for"), which the
// server also knows (glamour.rs).

const KAWARI_ROLE_GROUPS = ["Tanks", "Healers", "Melee DPS", "Physical Ranged DPS", "Caster DPS"];

// The dropdown's groups: [label, names] pairs, the roles first, then each role's classes from the server's list
// (`classList`: { roles, groups: [{ name, classes }] }, or none before connecting.)
function kawariClassGroups(classList) {
  const groups = [["Roles", (classList && classList.roles) || KAWARI_ROLE_GROUPS]];
  for (const group of (classList && classList.groups) || []) {
    if (group.classes && group.classes.length) groups.push([group.name, group.classes]);
  }
  return groups;
}

// Whether the server will match `name`: a role group, or a class or job (by name or abbreviation) from its list.
// Without the list nothing can be told, so every name passes.
function kawariKnownClass(classList, name) {
  if (!classList) return true;
  const wanted = name.trim().toLowerCase();
  if (["all", "all classes", "any", "any class"].includes(wanted)) return true;
  if (KAWARI_ROLE_GROUPS.some((role) => role.toLowerCase() === wanted)) return true;
  return (classList.classes || []).some(
    (entry) => entry.name.toLowerCase() === wanted || (entry.abbreviation || "").toLowerCase() === wanted,
  );
}

if (typeof module !== "undefined") {
  module.exports = { KAWARI_ROLE_GROUPS, kawariClassGroups, kawariKnownClass };
}
