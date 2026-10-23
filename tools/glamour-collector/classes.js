// The Class dropdown's choices: the role groups Eorzea Collection's "Intended for" uses, then each combat job and base
// class by role. Kawari's server matches these to its ClassJob sheet by English name (glamour.rs), and works the role
// groups out from the sheet's Role and PrimaryStat, so these are only the dropdown's labels. Eorzea Collection's own
// labels ("Gatherers", "Crafters"...) can still come in from a page; the server leaves those glamours out, as bots
// are all combat classes.

const KAWARI_CLASS_GROUPS = [
  ["Roles", ["Tanks", "Healers", "Melee DPS", "Physical Ranged DPS", "Caster DPS"]],
  ["Tanks", ["Paladin", "Warrior", "Dark Knight", "Gunbreaker", "Gladiator", "Marauder"]],
  ["Healers", ["White Mage", "Scholar", "Astrologian", "Sage", "Conjurer"]],
  ["Melee DPS", ["Monk", "Dragoon", "Ninja", "Samurai", "Reaper", "Viper", "Pugilist", "Lancer", "Rogue"]],
  ["Physical Ranged DPS", ["Bard", "Machinist", "Dancer", "Archer"]],
  ["Caster DPS", ["Black Mage", "Summoner", "Red Mage", "Pictomancer", "Blue Mage", "Thaumaturge", "Arcanist"]],
];

const KAWARI_CLASS_NAMES = KAWARI_CLASS_GROUPS.flatMap(([, names]) => names);

if (typeof module !== "undefined") {
  module.exports = { KAWARI_CLASS_GROUPS, KAWARI_CLASS_NAMES };
}
