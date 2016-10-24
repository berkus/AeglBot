package org.aeriagloris.telegram.services

// @todo Move this to DB "where arguments[0] in Activity.shortcuts"
object ActivityIndex {
    public val map = mapOf(
        "kf" to Pair("King's Fall", "hard"),
        "kfh" to Pair("King's Fall", "hard"),
        "kfn" to Pair("King's Fall", "normal"),
        "cr" to Pair("Crota's End", "hard"),
        "crh" to Pair("Crota's End", "hard"),
        "crn" to Pair("Crota's End", "normal"),
        "vog" to Pair("Vault of Glass", "hard"),
        "vogh" to Pair("Vault of Glass", "hard"),
        "vogn" to Pair("Vault of Glass", "normal"),
        "wotm" to Pair("Wrath of the Machine", "normal"),
        "wotmn" to Pair("Wrath of the Machine", "normal"),
        "wotmh" to Pair("Wrath of the Machine", "hard"),
        "pvp" to Pair("Crucible", "any"),
        "3v3" to Pair("Crucible", "3v3"),
        "6v6" to Pair("Crucible", "6v6"),
        "ib" to Pair("Crucible", "Iron Banner"),
        "too" to Pair("Crucible", "Trials of Osiris"),
        "pvt" to Pair("Crucible", "Private Matches"),
        "trn" to Pair("Crucible", "Private Tournament"),
        "pve" to Pair("Vanguard", "any"),
        "patrol" to Pair("Vanguard", "Patrols"),
        "coo" to Pair("Vanguard", "Court of Oryx"),
        "forge" to Pair("Vanguard", "Archon's Forge"),
        "poe" to Pair("Vanguard", "Prison of Elders"),
        "coe" to Pair("Vanguard", "Challenge of Elders")
        // poe 41, etc
    )
}
